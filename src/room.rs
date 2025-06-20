use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tracing::{info, debug, warn};

type Peer = tokio::sync::mpsc::UnboundedSender<Message>;

pub struct RoomMap(pub DashMap<String, Vec<Peer>>);

impl RoomMap {
    pub fn new() -> Self {
        Self(DashMap::new())
    }

    pub fn join(&self, room: &str, peer: Peer) {
        self.0.entry(room.to_string()).or_default().push(peer);
        info!("Peer joined room: {}", room);
    }

    pub fn broadcast(&self, room: &str, sender_id: usize, msg: Message) {
        if let Some(peers) = self.0.get(room) {
            debug!("Broadcasting message in room: {} from peer: {}", room, sender_id);
            for (i, peer) in peers.iter().enumerate() {
                if i != sender_id {
                    let _ = peer.send(msg.clone());
                    debug!(" → sent to peer {}", i);
                }
            }
        } else {
            warn!("Broadcast failed: room '{}' not found", room);
        }
    }
}

pub async fn handle_socket(stream: WebSocket, room_id: String, rooms: Arc<RoomMap>) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    rooms.join(&room_id, tx.clone());
    let my_id = rooms.0.get(&room_id).unwrap().len() - 1;
    info!("WebSocket peer assigned ID {} in room '{}'", my_id, room_id);

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            debug!("Sending message to peer {}: {:?}", my_id, msg);
            if sender.send(msg).await.is_err() {
                warn!("Failed to send message to peer {}", my_id);
                break;
            }
        }
        info!("Send loop ended for peer {}", my_id);
    });

    let recv_task = {
        let rooms = rooms.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                debug!("Peer {} received message: {:?}", my_id, msg);
                if matches!(msg, Message::Text(_) | Message::Binary(_)) {
                    rooms.broadcast(&room_id, my_id, msg);
                }
            }
            info!("Receive loop ended for peer {}", my_id);
        })
    };

    let _ = tokio::join!(send_task, recv_task);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::ws::Message;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn test_room_join_and_broadcast() {
        let rooms = RoomMap::new();

        // Создаём 3 тестовых клиента
        let (tx1, mut rx1) = unbounded_channel::<Message>();
        let (tx2, mut rx2) = unbounded_channel::<Message>();
        let (tx3, mut rx3) = unbounded_channel::<Message>();

        rooms.join("test_room", tx1);
        rooms.join("test_room", tx2);
        rooms.join("test_room", tx3);

        let msg = Message::Text("Hello peers".to_string());

        // "Клиент 1" посылает сообщение — индекс 0
        rooms.broadcast("test_room", 0, msg.clone());

        // Проверяем, что 2 и 3 получили сообщение
        assert_eq!(rx2.recv().await.unwrap(), msg);
        assert_eq!(rx3.recv().await.unwrap(), msg);

        // Проверяем, что клиент-отправитель ничего не получил
        assert!(rx1.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_broadcast_empty_room_does_nothing() {
        let rooms = RoomMap::new();
        let msg = Message::Text("Ping".into());

        // Комната не существует — должно молча пройти
        rooms.broadcast("no_room", 0, msg.clone());
        // если дошло до сюда — тест успешен
    }

    #[tokio::test]
    async fn test_multiple_rooms_isolated() {
        let rooms = RoomMap::new();

        let (tx1, mut rx1) = unbounded_channel::<Message>();
        let (tx2, mut rx2) = unbounded_channel::<Message>();

        rooms.join("room_a", tx1);
        rooms.join("room_b", tx2);

        let msg = Message::Text("Message".into());

        // Шлём в room_a
        rooms.broadcast("room_a", 0, msg.clone());

        // В room_b не должно ничего прийти
        assert!(rx2.try_recv().is_err());
        // В room_a только один участник, он отправитель — тоже ничего
        assert!(rx1.try_recv().is_err());
    }
}

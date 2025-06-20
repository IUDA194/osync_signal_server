# -------- Stage 1: Builder --------
FROM rust:1.83 as builder

WORKDIR /app
COPY . .

# Компилируем бинарник в release-режиме
RUN cargo build --release

# -------- Stage 2: Runtime --------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Копируем только бинарник
COPY --from=builder /app/target/release/webrtc-signal-server /usr/local/bin/webrtc-signal-server

# Запускаем бинарник
CMD ["webrtc-signal-server"]

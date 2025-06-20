# 🧱 Этап 1: Сборка приложения
FROM rust:1.83 as builder

# Установка рабочей директории
WORKDIR /app

# Кэшируем зависимости
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release && rm -r src

# Копируем исходники и пересобираем
COPY . .
RUN cargo build --release

# 🏃 Этап 2: Релиз-образ
FROM debian:bookworm-slim

# Установим только необходимые зависимости
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Копируем скомпилированный бинарник
COPY --from=builder /app/target/release/webrtc-signal-server /usr/local/bin/server

# Порт по умолчанию
EXPOSE 3000

# Запуск
CMD ["webrtc-signal-server"]


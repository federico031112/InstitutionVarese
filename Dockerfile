FROM rust:1.78 AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release

RUN rm -f target/release/deps/sedi_microservice*

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/il_tuo_progetto_rust /app/sedi_microservice

EXPOSE 3000

CMD ["./sedi_microservice"]
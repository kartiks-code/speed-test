FROM rust:1 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/petstore-actix /usr/local/bin/petstore-actix
EXPOSE 8080
CMD ["petstore-actix"]

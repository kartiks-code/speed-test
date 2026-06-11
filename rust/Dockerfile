FROM rust:latest

WORKDIR /app

COPY . .

RUN cargo build --example petstore-server-server

EXPOSE 8080

CMD ["/app/target/debug/examples/petstore-server-server"]

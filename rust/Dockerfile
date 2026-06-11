FROM rust:1

WORKDIR /app

COPY . .

RUN cargo build --release --example petstore-server-server

EXPOSE 8080

CMD ["/app/target/release/examples/petstore-server-server"]

FROM golang:1.25
WORKDIR /go/src

COPY . .

RUN go build -o petstore .

ENV GIN_MODE=release
EXPOSE 8080/tcp
ENTRYPOINT ["./petstore"]

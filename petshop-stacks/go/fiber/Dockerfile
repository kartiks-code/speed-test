FROM golang:1.23
WORKDIR /go/src
COPY . .
RUN go build -o petstore .
EXPOSE 8080/tcp
ENTRYPOINT ["./petstore"]

# Use the official Drogon builder image; libdrogon-dev is not reliably
# available in Ubuntu 24.04 apt, so we build on a known Drogon base.
FROM drogonframework/drogon:latest AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends nlohmann-json3-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cmake -B build \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_CXX_COMPILER=clang++-14 && \
    cmake --build build --parallel $(nproc)

# ---- runtime stage ----
FROM ubuntu:22.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libpq5 libssl3 zlib1g libuuid1 libc-ares2 \
        libjsoncpp25 libmariadb3 libsqlite3-0 libhiredis0.14 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/build/petstore-drogon /usr/local/bin/petstore-drogon

# Copy Drogon runtime libs from builder (they live in /usr/local/lib)
COPY --from=builder /usr/local/lib/libdrogon.so* /usr/local/lib/
COPY --from=builder /usr/local/lib/libtrantor.so* /usr/local/lib/
RUN ldconfig

EXPOSE 8080
ENV POSTGRES_HOST=speed-test-postgres
ENV POSTGRES_PORT=5432
ENV POSTGRES_DB=cpp-drogon
ENV POSTGRES_USER=myuser
ENV POSTGRES_PASSWORD=mypassword

CMD ["petstore-drogon"]

FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY voiddrop-server/ voiddrop-server/
COPY vb/ vb/
RUN cargo build --release --bin voiddrop-server

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/voiddrop-server /usr/local/bin/voiddrop-server
EXPOSE 9876
CMD ["voiddrop-server"]

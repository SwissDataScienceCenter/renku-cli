FROM rust:bookworm AS builder
WORKDIR /app
COPY . .
RUN apt update && apt install -y cmake musl-tools
RUN rustup target add x86_64-unknown-linux-musl && cargo build --release --bin rnk -F vendored-openssl -F vendored-zlib --target-dir /build --target=x86_64-unknown-linux-musl

FROM gcr.io/distroless/static
COPY --from=builder /build/x86_64-unknown-linux-musl/release/rnk /rnk
ENTRYPOINT ["/rnk"]

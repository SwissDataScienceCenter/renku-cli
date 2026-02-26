FROM rust:bookworm AS builder
WORKDIR /app
COPY . .
RUN apt update && apt install -y cmake
RUN cargo build --release --bin rnk -F vendored-openssl -F vendored-zlib --target-dir /build

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /build/release/rnk /rnk
ENTRYPOINT ["/rnk"]

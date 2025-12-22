FROM rust:1.89.0-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slimRUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/easynote .
# COPY .env .env
EXPOSE 8000
CMD ["./easynote"]
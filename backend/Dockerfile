FROM rust:1.98-bookworm AS builder
WORKDIR /build/backend
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home sara
WORKDIR /app/backend
COPY --from=builder /build/backend/target/release/the-sara-protocol /usr/local/bin/the-sara-protocol
COPY migrations ./migrations
USER sara
ENV HOST=0.0.0.0 PORT=3000 MIGRATIONS_DIR=/app/backend/migrations
EXPOSE 3000
HEALTHCHECK --interval=5s --timeout=5s --start-period=10s --retries=10 \
    CMD curl --fail --silent "http://127.0.0.1:${PORT}/health" || exit 1
CMD ["the-sara-protocol"]

# ---------- Stage 1: Build ----------
FROM rust:1.82-slim AS builder

WORKDIR /app

# Only copy manifest files first (caches dependencies)
COPY Cargo.toml Cargo.lock ./

# Create a dummy src/main.rs to let Cargo build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release

# Now copy real source – invalidates only this layer
COPY src ./src

# Build final optimized binary
RUN cargo build --release


# ---------- Stage 2: Final Image ----------
FROM alpine:3.20

# Create non-root user
RUN addgroup -S app && adduser -S app -G app

COPY --from=builder /app/target/release/smem-exporter /usr/local/bin/smem-exporter

RUN chmod +x /usr/local/bin/smem-exporter

USER app

EXPOSE 9215

ENTRYPOINT ["smem-exporter"]

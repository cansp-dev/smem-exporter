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
FROM ubuntu:24.04

# Install minimal dependencies and create non-root user
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd -r app && \
    useradd -r -g app app

COPY --from=builder /app/target/release/smem-exporter /usr/local/bin/smem-exporter

RUN chmod +x /usr/local/bin/smem-exporter

USER app

EXPOSE 9215

# Health check to verify the application starts correctly
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/smem-exporter", "--version"] || exit 1

ENTRYPOINT ["smem-exporter"]

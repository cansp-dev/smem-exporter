FROM alpine:latest

# Install dependencies
RUN apk add --no-cache libgcc

# Create non-root user
RUN addgroup -S app && adduser -S app -G app

# Copy binary (verwende das musl binary für bessere Kompatibilität)
COPY target/x86_64-unknown-linux-musl/release/smem-exporter /usr/local/bin/

# Switch to non-root user
USER app

# Expose port
EXPOSE 9215

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:9215/health || exit 1

# Run the binary
CMD ["smem-exporter"]

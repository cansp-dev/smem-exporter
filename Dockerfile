FROM alpine:latest

RUN addgroup -S app && adduser -S app -G app
USER app

# Binary vom GitHub Release holen
# TAG wird beim Build gesetzt
COPY smem-exporter /usr/local/bin/smem-exporter

RUN chmod +x /usr/local/bin/smem-exporter

EXPOSE 9215

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:9215/health || exit 1

CMD ["/usr/local/bin/smem-exporter"]

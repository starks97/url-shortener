# Build stage
FROM rust:bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Final run stage
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Copia el binario desde la etapa de construcción
COPY --from=builder /app/target/release/url_shortener_api /app/url_shortener_api

# Ejecuta el binario
CMD ["/app/url_shortener_api"]

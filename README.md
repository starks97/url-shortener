# Url_Shortener Project



## Cargo watcher
       cargo watch -q -c -w src/ -x run


## up migrations
       sqlx migration run

       # Build stage
FROM rust:bookworm AS builder

WORKDIR /app

# Copia el archivo .env al contenedor en la etapa de construcción
COPY .env ./
COPY . .

RUN cargo build --release

# Final run stage
FROM debian:bookworm-slim AS runner

WORKDIR /app

# Install necessary libraries, including libssl
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copia el archivo .env a la etapa final
COPY --from=builder /app/.env ./

# Copia el binario construido desde la etapa de construcción
COPY --from=builder /app/target/release/url_shortener_api /app/url_shortener_api

CMD ["/app/url_shortener_api"]

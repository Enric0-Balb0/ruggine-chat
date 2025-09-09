# =========================
# Stage 1: Build Rust binary
# =========================
FROM rust:1.88-alpine AS builder

# Install dependencies per la compilazione
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    git

# Aggiungi target musl per build statica
RUN rustup target add x86_64-unknown-linux-musl

# Imposta cartella di lavoro
WORKDIR /app

# Copia Cargo.toml e Cargo.lock per caching layer
COPY Cargo.toml Cargo.lock ./

# Copia il codice sorgente
COPY src/ ./src/

# Abilita build statica e ignora warning
ENV RUSTFLAGS="-C target-feature=+crt-static -Awarnings"

# Compila in release
RUN cargo build --release --target x86_64-unknown-linux-musl

# =========================
# Stage 2: Runtime minimale
# =========================
FROM alpine:3.18

# Runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    tzdata \
    wget

# Crea utente non-root
RUN adduser -D -s /bin/sh ruggine

# Imposta cartella di lavoro
WORKDIR /app

# Copia il binario dalla build
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/ruggine_server /app/ruggine_server

# Copia il file .env specifico per Docker
COPY .env /app/.env

# Cambia ownership del binario e file .env
RUN chown -R ruggine:ruggine /app

# Passa all'utente non-root
USER ruggine

# Porta esposta
EXPOSE 8002

# Healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8002/health || exit 1

# Variabile per DB (da settare in runtime)
ENV DATABASE_URL=postgres://user:password@host:5432/dbname

# Comando di avvio
CMD ["./ruggine_server"]

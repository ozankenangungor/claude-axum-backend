# --- Stage 1: Builder ---
# Rust derleme ortamını kur - nightly edition2024 support için
FROM rust:nightly AS builder

# Çalışma dizinini oluştur
WORKDIR /usr/src/app

# Bağımlılıkları önbelleğe almak için önce sadece Cargo dosyalarını kopyala
COPY Cargo.toml Cargo.lock ./

# Dummy bir main.rs oluşturarak sadece bağımlılıkları derle
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Şimdi asıl kodumuzu kopyala
COPY src ./src
COPY build.rs ./

# migrations dizinini de kopyala (sqlx compile-time kontrolü için)
COPY migrations ./migrations

# SQLx offline mode için sqlx-data.json varsa kopyala
COPY sqlx-data.json* ./

# Uygulamayı release modunda derle
RUN rm -f target/release/deps/todo_api*
RUN cargo build --release

# --- Stage 2: Final Image ---
# Minimal bir işletim sistemi imajı kullan
FROM debian:bookworm-slim

# Güvenlik ve güncellemeler
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Derlenmiş binary'yi builder aşamasından kopyala
COPY --from=builder /usr/src/app/target/release/todo_api /usr/local/bin/todo_api

# Cloud Run için PORT ortam değişkenini tanımla (varsayılan olarak)
ENV PORT=8080

# Uygulamayı çalıştıracak komut
CMD ["todo_api"]
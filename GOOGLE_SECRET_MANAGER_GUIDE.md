# 🔐 Google Cloud Run + Neon PostgreSQL Konfigürasyon Rehberi

## 🎯 Özet

Projeniz artık **Google Cloud Run** ve **Neon PostgreSQL** için optimize edildi:

- ✅ **Production**: Google Secret Manager kullanır (güvenli)
- ✅ **Development**: Environment variables kullanır (kolay)
- ✅ **Auto-detection**: Ortamı otomatik tespit eder

## 🚀 Nasıl Çalışır?

### Production (Cloud Run + Neon)
```rust
// main.rs içinde
let config = Config::auto_load().await?;

// Otomatik olarak şunu tespit eder:
// - RUST_ENV=production
// - GCP_PROJECT_ID mevcut
// - K_SERVICE mevcut (Cloud Run)
// → Google Secret Manager kullanır
```

### Development (Local)
```rust
// Aynı kod farklı ortamda
let config = Config::auto_load().await?;

// Otomatik olarak şunu tespit eder:
// - RUST_ENV != production VEYA
// - GCP_PROJECT_ID yok
// → Environment variables kullanır
```

## 🔑 Secret Management

### Production'da (Secret Manager)
Hassas bilgiler Google Secret Manager'da saklanır:

| Secret Name | Açıklama |
|-------------|----------|
| `database-url` | Neon PostgreSQL connection string |
| `jwt-secret` | JWT token signing key (min 32 char) |
| `hashing-secret` | Password hashing key (min 16 char) |

### Development'da (Environment Variables)
Local development için `.env` dosyası:

```bash
DATABASE_URL=postgresql://localhost:5432/todoapp
JWT_SECRET=your-development-jwt-secret-32-chars
HASHING_SECRET_KEY=your-dev-hashing-16char
```

## 🌍 Environment Variables

### Cloud Run Production
```bash
# Gerekli - Secret Manager için
GCP_PROJECT_ID=your-gcp-project-id
RUST_ENV=production

# Opsiyonel - Neon optimizasyonu için
NEON_BRANCH_NAME=main
NEON_COMPUTE_ENDPOINT=ep-xxx.region.aws.neon.tech

# Application settings
PORT=8080
HOST=0.0.0.0
RUST_LOG=info
```

### Local Development
```bash
# Secrets (.env dosyasında)
DATABASE_URL=postgresql://user:pass@localhost:5432/db
JWT_SECRET=development-secret-at-least-32-characters
HASHING_SECRET_KEY=dev-hashing-secret-16

# Application settings
PORT=8080
RUST_LOG=debug
```

## 🔧 Deployment

### 1. Google Secret Manager'da Secrets Oluştur

```bash
# Database URL
echo -n "postgresql://user:pass@ep-xxx.neon.tech/db?sslmode=require" | \
    gcloud secrets create database-url --data-file=-

# JWT Secret  
openssl rand -base64 32 | \
    gcloud secrets create jwt-secret --data-file=-

# Hashing Secret
openssl rand -base64 24 | \
    gcloud secrets create hashing-secret --data-file=-
```

### 2. Cloud Run'a Deploy Et

```bash
# Deployment script'i çalıştır
./deploy-cloudrun-neon.sh
```

### 3. Environment Variables Ayarla

Cloud Run service'te şu environment variables'ları tanımlayın:

```bash
GCP_PROJECT_ID=your-project-id
RUST_ENV=production
PORT=8080
RUST_LOG=info
```

## 🧪 Test Etme

### Local Development Test
```bash
# .env dosyasını ayarla
cp .env.staging .env

# Uygulamayı çalıştır  
cargo run

# Log'da görmeli:
# "Development environment detected - using environment variables"
```

### Production Test
```bash
# Environment variables ayarla
export GCP_PROJECT_ID=your-project-id
export RUST_ENV=production

# Uygulamayı çalıştır
cargo run

# Log'da görmeli:
# "Production/Cloud Run environment detected - using Google Secret Manager"
```

## 🛡️ Güvenlik

### Production Güvenliği ✅
- JWT secrets Secret Manager'da
- Database credentials Secret Manager'da  
- Environment variables'da hassas bilgi yok
- Auto-rotation support

### Development Kolaylığı ✅
- Local .env file support
- Hızlı development cycle
- Test friendly configuration

## 📝 Önemli Notlar

1. **Secret Names**: Google Secret Manager'da secret isimleri sabit:
   - `database-url`
   - `jwt-secret` 
   - `hashing-secret`

2. **Auto-Detection**: Ortam otomatik tespit edilir:
   - `RUST_ENV=production` → Secret Manager
   - `K_SERVICE` var → Secret Manager (Cloud Run)
   - Diğer durumlarda → Environment variables

3. **Neon Optimization**: Connection pooling Neon serverless için optimize edildi

4. **Error Handling**: Secret Manager erişim hatalarında güvenli fallback

## 🚨 Sorun Giderme

### "Secret not found" hatası
```bash
# Secret'in var olduğunu kontrol et
gcloud secrets list

# Secret'i oluştur
echo -n "your-secret-value" | gcloud secrets create secret-name --data-file=-
```

### "Permission denied" hatası  
```bash
# Service account'a Secret Manager erişimi ver
gcloud projects add-iam-policy-binding PROJECT_ID \
   --member="serviceAccount:SERVICE_ACCOUNT@PROJECT_ID.iam.gserviceaccount.com" \
   --role="roles/secretmanager.secretAccessor"
```

### Development'da Secret Manager kullanmak
```bash
# GCP_PROJECT_ID'yi kaldır veya boş bırak
unset GCP_PROJECT_ID

# Veya RUST_ENV'i development yap
export RUST_ENV=development
```

## ✨ Sonuç

Artık projeniz:
- 🔐 Production'da Google Secret Manager kullanıyor
- 🏠 Development'da environment variables kullanıyor  
- 🤖 Ortamı otomatik tespit ediyor
- 🚀 Cloud Run + Neon için optimize edildi

Hem güvenli hem de developer-friendly! 🎉
# Google Cloud Run Deployment Guide

Bu döküman, Rust Axum TODO API'sini Google Cloud Run'da deploy etmek için gerekli tüm adımları içerir.

## Önemli Notlar ⚠️

### Environment Variables (Ortam Değişkenleri)
Cloud Run servisinde aşağıdaki ortam değişkenlerini ayarlayın:

```bash
DATABASE_URL=postgres://username:password@host:port/database
JWT_SECRET=your-jwt-secret-key-min-32-chars
HASHING_SECRET_KEY=your-hashing-secret-min-16-chars
FRONTEND_URL=https://your-frontend-domain.com  # CORS için
```

### Gizli Bilgilerin Yönetimi
Production ortamında gizli bilgileri Google Secret Manager ile yönetin:

1. Google Cloud Console'da Secret Manager servisine gidin
2. Her gizli bilgi için ayrı secret oluşturun:
   - `database-url`
   - `jwt-secret`
   - `hashing-secret`
3. Cloud Run servisini oluştururken bu secret'lara erişim iznini verin

## Deployment Adımları

### 1. Projeyi Cloud Build ile Build Etme

```bash
# Project ID'nizi ayarlayın
export PROJECT_ID=your-project-id

# Docker image'ı build edin
gcloud builds submit --tag gcr.io/$PROJECT_ID/todo-api

# Veya Cloud Build config dosyası ile:
gcloud builds submit --config cloudbuild.yaml
```

### 2. Database Migration Çalıştırma

```bash
# SQLx CLI'yi yükleyin (henüz yoksa)
cargo install sqlx-cli

# Migration'ları çalıştırın
sqlx migrate run --database-url $DATABASE_URL
```

### 3. Cloud Run Service Oluşturma

```bash
gcloud run deploy todo-api \
  --image gcr.io/$PROJECT_ID/todo-api \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --port 8080 \
  --memory 512Mi \
  --concurrency 80 \
  --timeout 300 \
  --set-env-vars="FRONTEND_URL=https://your-frontend.com" \
  --set-secrets="/secrets/database-url=database-url:latest,/secrets/jwt-secret=jwt-secret:latest,/secrets/hashing-secret=hashing-secret:latest"
```

## Performans Optimizasyonları

### Container Optimizasyonları
- Multi-stage build kullanıldı: Image boyutu ~50MB
- Debian Slim base image: Güvenlik ve boyut optimizasyonu
- CA sertifikaları dahil: HTTPS bağlantıları için

### Application Optimizasyonları
- JSON structured logging: Cloud Logging entegrasyonu
- Connection pooling: PostgreSQL bağlantı havuzu
- Request timeout: 30 saniye
- Body limit: 1MB
- Compression: Gzip sıkıştırma

### Cloud Run Optimizasyonları
- Minimum instances: 0 (soğuk başlangıç)
- Maximum instances: 100 (otomatik ölçeklendirme)
- Concurrency: 80 request/instance
- Memory: 512Mi (gerekirse artırılabilir)
- CPU: 1 vCPU (gerekirse artırılabilir)

## Monitoring ve Debugging

### Logs Görüntüleme
```bash
# Cloud Run loglarını görüntüle
gcloud logging read "resource.type=cloud_run_revision" --limit 100 --format json

# Specific service logs
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=todo-api" --limit 50
```

### Health Check
Service URL'inizde şu endpoint'leri test edin:
- `GET /auth/register` (405 Method Not Allowed beklenir - bu normal)
- `POST /auth/register` (kayıt için)
- `POST /auth/login` (giriş için)

## Güvenlik

### CORS Configuration
- Production'da `FRONTEND_URL` ortam değişkeniyle frontend domain'inizi ayarlayın
- Wildcard (`*`) sadece development için kullanılmalı

### Secret Management
- Asla gizli bilgileri kod repository'sinde saklamayın
- Google Secret Manager kullanın
- Regular secret rotation yapın

### Network Security
- Cloud Run servisi varsayılan olarak HTTPS'i zorlar
- Cloud Armor ile DDoS koruması ekleyebilirsiniz
- VPC connector ile private network erişimi sağlayabilirsiniz

## Troubleshooting

### Common Issues

1. **Port Binding Error**
   - Cloud Run PORT env variable'ını kontrol edin
   - Application'ın 0.0.0.0'a bind ettiğinden emin olun

2. **Database Connection Issues**
   - DATABASE_URL'in doğru formatda olduğunu kontrol edin
   - Cloud SQL Proxy kullanıyorsanız connection string'i güncelleyin

3. **Cold Start Performance**
   - Minimum instances > 0 ayarlayın
   - Initialization time'ı optimize edin

4. **Memory Issues**
   - Memory limitini artırın (512Mi → 1Gi)
   - Connection pool boyutunu kontrol edin

### Health Monitoring
```bash
# Service status
gcloud run services describe todo-api --region=us-central1

# Traffic allocation
gcloud run services list --filter="metadata.name=todo-api"
```

## CI/CD Pipeline (Önerilen)

Cloud Build configuration örneği (`cloudbuild.yaml`):

```yaml
steps:
  # Run database migrations
  - name: 'gcr.io/cloud-builders/gcloud'
    entrypoint: 'bash'
    args:
      - '-c'
      - |
        # Install sqlx-cli
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
        cargo install sqlx-cli
        # Run migrations
        sqlx migrate run --database-url $$DATABASE_URL
    secretEnv: ['DATABASE_URL']

  # Build the container image
  - name: 'gcr.io/cloud-builders/docker'
    args: ['build', '-t', 'gcr.io/$PROJECT_ID/todo-api', '.']

  # Push the container image
  - name: 'gcr.io/cloud-builders/docker'
    args: ['push', 'gcr.io/$PROJECT_ID/todo-api']

  # Deploy to Cloud Run
  - name: 'gcr.io/cloud-builders/gcloud'
    args:
      - 'run'
      - 'deploy'
      - 'todo-api'
      - '--image'
      - 'gcr.io/$PROJECT_ID/todo-api'
      - '--region'
      - 'us-central1'
      - '--platform'
      - 'managed'
      - '--allow-unauthenticated'

availableSecrets:
  secretManager:
    - versionName: projects/$PROJECT_ID/secrets/database-url/versions/latest
      env: 'DATABASE_URL'

images:
  - 'gcr.io/$PROJECT_ID/todo-api'
```

Bu guide'ı takip ederek uygulamanız production-ready bir şekilde Cloud Run'da çalışacaktır.
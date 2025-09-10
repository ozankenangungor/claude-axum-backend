# 🚀 PRODUCTION DEPLOYMENT - STEP BY STEP GUIDE

## 🎯 ÖZET: Kod Değişikliğini Production'a Nasıl Gönderir?

### ⚡ EN KOLAY YOL (Recommended)
```bash
git add .
git commit -m "feat: yeni özellik eklendi"
git push origin main
```
**Bu kadar! GitHub Actions otomatik olarak:**
1. ✅ Testleri çalıştırır
2. ✅ Code quality check yapar  
3. ✅ Docker build yapar
4. ✅ Production'a deploy eder
5. ✅ Health check yapar

---

## 🏗️ FULL SETUP (One-time kurulum)

### 1. 📁 Repository Hazırlığı
```bash
# Repository'nizi GitHub'a push edin
git remote add origin https://github.com/YOUR_USERNAME/todo_api.git
git push -u origin main
```

### 2. 🔐 Google Cloud Setup
```bash
# Service Account oluştur
gcloud iam service-accounts create github-actions \
    --description="GitHub Actions deployment" \
    --display-name="GitHub Actions"

# İzinleri ver
gcloud projects add-iam-policy-binding YOUR_PROJECT_ID \
    --member="serviceAccount:github-actions@YOUR_PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/run.admin"

gcloud projects add-iam-policy-binding YOUR_PROJECT_ID \
    --member="serviceAccount:github-actions@YOUR_PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/storage.admin"

# Key oluştur
gcloud iam service-accounts keys create key.json \
    --iam-account=github-actions@YOUR_PROJECT_ID.iam.gserviceaccount.com
```

### 3. 🔑 GitHub Secrets Ayarlama
GitHub Repository'de: **Settings > Secrets and variables > Actions**

**Ekleyeceğiniz secrets:**
```
GCP_PROJECT_ID = your-project-id
GCP_SA_KEY = (key.json dosyasının tüm içeriğini kopyala-yapıştır)
```

### 4. 📦 Production Secrets (Google Secret Manager)
```bash
# Database URL
echo -n "postgres://user:pass@host:5432/db" | gcloud secrets create database-url --data-file=-

# JWT Secret (min 32 karakter)
echo -n "your-super-secure-jwt-secret-minimum-32-characters-long" | gcloud secrets create jwt-secret --data-file=-

# Hashing Secret (min 16 karakter)  
echo -n "your-hashing-secret-minimum-16-characters" | gcloud secrets create hashing-secret --data-file=-
```

---

## 🔄 GÜNLÜK WORKFLOW

### 🌟 Feature Development
```bash
# 1. Feature branch oluştur
git checkout -b feature/yeni-ozellik

# 2. Değişiklik yap
# ... kod değişiklikleri ...

# 3. Local test et
cargo test
cargo fmt
cargo clippy

# 4. Commit et
git add .
git commit -m "feat: sosyal medya özelliği eklendi"

# 5. Push et  
git push origin feature/yeni-ozellik

# 6. GitHub'da Pull Request oluştur
# 7. Review'dan sonra main'e merge et
# 8. 🚀 Otomatik production deployment başlar!
```

### 🚨 Hotfix (Acil Düzeltme)
```bash
# 1. Hotfix branch
git checkout -b hotfix/critical-bug-fix

# 2. Hızlı düzeltme
# ... fix code ...

# 3. Test et
cargo test

# 4. Direkt main'e merge et
git checkout main
git merge hotfix/critical-bug-fix
git push origin main

# 5. 🚀 Otomatik deployment (2-3 dakika)
```

---

## 🌍 MULTI-ENVIRONMENT STRATEGY

### Branch Strategy
```
main branch     → 🚀 Production deployment
develop branch  → 🧪 Staging deployment  
feature/*       → 💻 Development deployment
```

### Environment URLs
- **Production:** `https://todo-api-xxx.run.app`
- **Staging:** `https://todo-api-staging-xxx.run.app`
- **Development:** `https://todo-api-dev-123-xxx.run.app`

---

## 📊 MONITORING & DEBUGGING

### 🔍 Deployment Status Check
```bash
# GitHub Actions'da deployment durumunu kontrol et
# Repository > Actions sekmesi

# Cloud Run'da service durumu
gcloud run services describe todo-api --region us-central1

# Logs
gcloud logs read --service=todo-api --limit=50
```

### 🏥 Health Check
```bash
# Production health check
curl https://your-service-url.run.app/health

# Expected response:
# {"status":"healthy","timestamp":"2024-...","service":"todo_api"}
```

### 🚨 Rollback (Acil Durum)
```bash
# Previous version'a geri dön
gcloud run services update-traffic todo-api \
  --to-revisions=PREVIOUS=100 \
  --region us-central1
```

---

## ⚡ HIZLI KOMUTLAR

### 📱 Development
```bash
cargo run              # Local server başlat
cargo test             # Testleri çalıştır
cargo fmt              # Code format
cargo clippy           # Linting
```

### 🚀 Production Deploy
```bash
git push origin main   # Otomatik deployment tetikle
```

### 🔧 Manuel Deploy (Acil)
```bash
docker build -f Dockerfile.production -t gcr.io/$PROJECT_ID/todo-api .
docker push gcr.io/$PROJECT_ID/todo-api
gcloud run deploy todo-api --image gcr.io/$PROJECT_ID/todo-api --region us-central1
```

---

## 🎯 BEST PRACTICES

### ✅ DO's
- ✅ Her değişiklik için test yaz
- ✅ Feature branch kullan
- ✅ Meaningful commit messages
- ✅ Pull request ile review yap
- ✅ Health check endpoint'ini test et

### ❌ DON'Ts  
- ❌ Direkt main'e push etme (hotfix hariç)
- ❌ Test'siz kod deploy etme
- ❌ Secret'ları kod'a yazma
- ❌ Production'da debug log bırakma

---

## 🎉 SONUÇ

**Artık modern bir CI/CD pipeline'ınız var!**

1. **Kod yaz** → `git push origin main`
2. **GitHub Actions çalışır** → Otomatik test + deploy
3. **Production'da live!** → 2-3 dakika içinde

**Bu kadar basit! 🚀**

**Deployment URL:** Pipeline tamamlandığında GitHub Actions size production URL'ini verecek.

**Next Level:** Monitoring, alerting, ve advanced deployment strategies ekleyebilirsiniz.
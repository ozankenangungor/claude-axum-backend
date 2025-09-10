# 🚀 GitHub Actions Secrets Setup Guide

## 📋 Adım Adım Secret Kurulumu

### 1. 🔐 Google Cloud Service Account Oluşturma

```bash
# 1. Service account oluştur
gcloud iam service-accounts create github-actions \
    --description="GitHub Actions deployment service account" \
    --display-name="GitHub Actions Deployer"

# 2. Project ID'nizi alın
export PROJECT_ID=$(gcloud config get-value project)
echo "Project ID: $PROJECT_ID"

# 3. Gerekli rolleri atayın
gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:github-actions@$PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/run.admin"

gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:github-actions@$PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/storage.admin"

gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:github-actions@$PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/iam.serviceAccountUser"

gcloud projects add-iam-policy-binding $PROJECT_ID \
    --member="serviceAccount:github-actions@$PROJECT_ID.iam.gserviceaccount.com" \
    --role="roles/secretmanager.secretAccessor"

# 4. Service account key oluşturun
gcloud iam service-accounts keys create github-actions-key.json \
    --iam-account=github-actions@$PROJECT_ID.iam.gserviceaccount.com

echo "✅ Service account key created: github-actions-key.json"
```

### 2. 📁 GitHub Repository Secret'ları

GitHub repository'nizde: **Settings > Secrets and variables > Actions > New repository secret**

#### Secret 1: GCP_PROJECT_ID
```
Name: GCP_PROJECT_ID
Value: [YOUR_PROJECT_ID]
```

#### Secret 2: GCP_SA_KEY  
```
Name: GCP_SA_KEY
Value: [github-actions-key.json dosyasının TÜM içeriği]
```

**Önemli:** `github-actions-key.json` dosyasını açın ve tüm JSON içeriğini kopyalayın:
```json
{
  "type": "service_account",
  "project_id": "your-project",
  "private_key_id": "...",
  "private_key": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n",
  "client_email": "github-actions@your-project.iam.gserviceaccount.com",
  "client_id": "...",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "..."
}
```

### 3. 🔑 Google Secret Manager Setup

```bash
# Production secret'larını oluşturun
echo -n "postgres://user:password@host:5432/database" | \
    gcloud secrets create database-url --data-file=-

echo -n "your-super-secure-jwt-secret-minimum-32-characters-long-for-production" | \
    gcloud secrets create jwt-secret --data-file=-

echo -n "your-hashing-secret-minimum-16-characters-for-production" | \
    gcloud secrets create hashing-secret --data-file=-

# Verify secrets created
gcloud secrets list
```

### 4. ✅ Test Setup

#### Test 1: Secret'ların Varlığını Kontrol Et
```bash
# GitHub'da secret'ların eklendiğini kontrol et
# Repository > Settings > Secrets and variables > Actions
# GCP_PROJECT_ID ve GCP_SA_KEY görünmeli
```

#### Test 2: Service Account Test
```bash
# Local'de service account'u test et
gcloud auth activate-service-account \
    --key-file=github-actions-key.json

gcloud projects list
# Project'inizi görmelisiniz
```

#### Test 3: Secret Manager Access Test
```bash
# Secret'lara erişimi test et
gcloud secrets versions access latest --secret="database-url"
gcloud secrets versions access latest --secret="jwt-secret"
gcloud secrets versions access latest --secret="hashing-secret"
```

### 5. 🚀 İlk Deployment Test

```bash
# Repository'nizi güncelleyin
git add .
git commit -m "feat: setup GitHub Actions deployment"
git push origin main

# GitHub Actions'ı kontrol edin
# Repository > Actions sekmesi
# Workflow çalışmalı ve başarılı olmalı
```

## 🔍 Troubleshooting

### Problem 1: "Invalid credentials" hatası
**Çözüm:** GCP_SA_KEY secret'ını tekrar kontrol et, JSON formatının doğru olduğundan emin ol

### Problem 2: "Permission denied" hatası  
**Çözüm:** Service account role'lerini kontrol et:
```bash
gcloud projects get-iam-policy $PROJECT_ID \
    --flatten="bindings[].members" \
    --filter="bindings.members:github-actions@$PROJECT_ID.iam.gserviceaccount.com"
```

### Problem 3: "Secret not found" hatası
**Çözüm:** Secret Manager'da secret'ların oluştuğunu kontrol et:
```bash
gcloud secrets list
```

### Problem 4: "Project not found" hatası
**Çözüm:** PROJECT_ID secret'ının doğru olduğunu kontrol et

## 📊 Final Checklist

Deployment'tan önce kontrol edin:

- [ ] ✅ Google Cloud Service Account oluşturuldu
- [ ] ✅ Service Account'a gerekli roller atandı  
- [ ] ✅ Service Account key JSON'u oluşturuldu
- [ ] ✅ GitHub'da GCP_PROJECT_ID secret'ı eklendi
- [ ] ✅ GitHub'da GCP_SA_KEY secret'ı eklendi (full JSON)
- [ ] ✅ Google Secret Manager'da production secret'lar oluşturuldu
- [ ] ✅ Local'de service account test edildi
- [ ] ✅ Secret Manager access test edildi

**Hepsi ✅ ise artık `git push origin main` yapabilirsiniz! 🚀**

## 🔒 Güvenlik Best Practices

1. **Key Rotation:** Service account key'lerini düzenli olarak yenileyin
2. **Minimal Permissions:** Sadece gerekli role'leri verin
3. **Secret Monitoring:** Secret access'lerini Google Cloud Console'da takip edin
4. **Backup:** Secret'ları güvenli bir yerde yedekleyin
5. **Access Control:** GitHub repository access'ini sınırlayın

---

**Bu setup'ı tamamladıktan sonra deployment tamamen otomatik olacak! 🎉**
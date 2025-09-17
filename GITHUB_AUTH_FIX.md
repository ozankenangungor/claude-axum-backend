# 🔧 GitHub Actions Google Cloud Authentication Fix

## ❌ **PROBLEM**
```
Error: google-github-actions/auth failed with: failed to generate Google Cloud federated token
The target service indicated by the "audience" parameters is invalid.
```

**Root Cause**: Workload Identity Provider not configured properly

## ✅ **SOLUTION: Use Service Account Key Authentication**

### Step 1: Create Service Account Key
```bash
# Create service account key (only if not exists)
gcloud iam service-accounts keys create ~/github-actions-key.json \
    --iam-account=github-actions@velvety-matter-471516-n4.iam.gserviceaccount.com

# Display the key (copy this JSON)
cat ~/github-actions-key.json
```

### Step 2: Add GitHub Secret
1. Go to: https://github.com/ozankenangungor/claude-axum-backend/settings/secrets/actions
2. Click **"New repository secret"**
3. Name: `GCP_SA_KEY`
4. Value: **Paste the entire JSON content from step 1**
5. Click **"Add secret"**

### Step 3: Verify GitHub Actions Update
The workflow is now updated to use:
```yaml
- name: Google Auth
  uses: 'google-github-actions/auth@v2'
  with:
    credentials_json: '${{ secrets.GCP_SA_KEY }}'  # ✅ Simple & reliable
```

Instead of the complex Workload Identity:
```yaml
# ❌ Old (Failed)
with:
  service_account: 'github-actions@...'
  workload_identity_provider: 'projects/364661851580/...'
```

## 🚀 **NEXT STEPS**

1. **Create service account key** (run commands above)
2. **Add GCP_SA_KEY secret** to GitHub repository
3. **Push code** - GitHub Actions should work
4. **Monitor deployment** - should deploy to Cloud Run successfully

## 🔒 **SECURITY NOTES**

- Service Account Key is temporary solution
- For production, Workload Identity is recommended
- Key should be rotated regularly
- Only minimum required permissions granted

## 📋 **ALTERNATIVE: Manual Deployment**

If GitHub Actions still fails, you can deploy manually:
```bash
# Local deployment (as backup)
./deploy-cloudrun-neon.sh
```

## ✅ **EXPECTED RESULT**

After adding the secret, GitHub Actions should:
1. ✅ Authenticate successfully
2. ✅ Build Docker image
3. ✅ Push to Container Registry  
4. ✅ Deploy to Cloud Run
5. ✅ Health check pass

**Service URL**: https://todo-api-364661851580.us-central1.run.app
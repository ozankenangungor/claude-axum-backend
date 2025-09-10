# 🚀 Quick Deployment Commands

## Local Development
```bash
# Start development server
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Check for issues
cargo clippy
```

## Production Deployment (Automated)
```bash
# Simple deployment - just push to main!
git add .
git commit -m "feat: your changes"
git push origin main

# GitHub Actions will automatically:
# 1. Run tests ✅
# 2. Build Docker image 🐳
# 3. Deploy to Cloud Run 🚀
# 4. Run health checks 🏥
```

## Manual Deployment (Emergency)
```bash
# Build and deploy manually
docker build -f Dockerfile.production -t gcr.io/$PROJECT_ID/todo-api .
docker push gcr.io/$PROJECT_ID/todo-api
gcloud run deploy todo-api --image gcr.io/$PROJECT_ID/todo-api --region us-central1
```

## Environment URLs
- **Development:** http://localhost:8080
- **Staging:** https://staging-todo-api-hash.run.app
- **Production:** https://todo-api-hash.run.app

## Health Checks
```bash
# Check if service is running
curl https://your-service-url.run.app/health

# Expected response:
# {"status":"healthy","timestamp":"2024-...","service":"todo_api"}
```
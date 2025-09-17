#!/bin/bash

# Google Cloud Run + Neon PostgreSQL Deployment Script
# Production-ready deployment with comprehensive configuration

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ID=${GCP_PROJECT_ID:-"velvety-matter-471516-n4"}
SERVICE_NAME=${SERVICE_NAME:-"todo-api"}
REGION=${REGION:-"us-central1"}
NEON_DATABASE_URL=${NEON_DATABASE_URL:-""}
NEON_BRANCH=${NEON_BRANCH:-"main"}

# Service account for production
SERVICE_ACCOUNT_EMAIL=${SERVICE_ACCOUNT:-"todo-api@${PROJECT_ID}.iam.gserviceaccount.com"}

echo -e "${BLUE}🚀 Starting Google Cloud Run + Neon PostgreSQL Deployment${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check prerequisites
echo -e "${BLUE}📋 Checking prerequisites...${NC}"

# Check if gcloud is installed and authenticated
if ! command -v gcloud &> /dev/null; then
    print_error "Google Cloud CLI is not installed"
    exit 1
fi

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed"
    exit 1
fi

# Set GCP project
echo "Setting GCP project to: $PROJECT_ID"
gcloud config set project $PROJECT_ID

# Enable required APIs
echo -e "${BLUE}🔧 Enabling required Google Cloud APIs...${NC}"
gcloud services enable \
    cloudbuild.googleapis.com \
    run.googleapis.com \
    secretmanager.googleapis.com \
    logging.googleapis.com \
    monitoring.googleapis.com

print_status "APIs enabled successfully"

# Build and deploy using Cloud Build
echo -e "${BLUE}🏗️  Building and deploying with Cloud Build...${NC}"

# Create cloudbuild.yaml if it doesn't exist
if [ ! -f "cloudbuild.yaml" ]; then
    echo "Creating Cloud Build configuration..."
    cat > cloudbuild.yaml << 'EOF'
steps:
  # Build the container image
  - name: 'gcr.io/cloud-builders/docker'
    args: 
      - 'build'
      - '-t'
      - 'gcr.io/$PROJECT_ID/todo-api:$COMMIT_SHA'
      - '-t'
      - 'gcr.io/$PROJECT_ID/todo-api:latest'
      - '.'

  # Push the container image to Container Registry
  - name: 'gcr.io/cloud-builders/docker'
    args:
      - 'push'
      - 'gcr.io/$PROJECT_ID/todo-api:$COMMIT_SHA'

  # Deploy to Cloud Run
  - name: 'gcr.io/google.com/cloudsdktool/cloud-sdk'
    entrypoint: gcloud
    args:
      - 'run'
      - 'deploy'
      - 'todo-api'
      - '--image'
      - 'gcr.io/$PROJECT_ID/todo-api:$COMMIT_SHA'
      - '--region'
      - 'us-central1'
      - '--platform'
      - 'managed'
      - '--allow-unauthenticated'
      - '--memory'
      - '1Gi'
      - '--cpu'
      - '2'
      - '--max-instances'
      - '100'
      - '--min-instances'
      - '1'
      - '--timeout'
      - '300'
      - '--concurrency'
      - '80'
      - '--service-account'
      - 'todo-api@$PROJECT_ID.iam.gserviceaccount.com'

images:
  - 'gcr.io/$PROJECT_ID/todo-api:$COMMIT_SHA'
  - 'gcr.io/$PROJECT_ID/todo-api:latest'

options:
  logging: CLOUD_LOGGING_ONLY
  machineType: 'E2_HIGHCPU_8'
EOF
fi

# Create or update secrets in Secret Manager
echo -e "${BLUE}🔐 Managing secrets in Google Secret Manager...${NC}"

# Function to create or update secret
create_or_update_secret() {
    local secret_name=$1
    local secret_value=$2
    
    if gcloud secrets describe $secret_name >/dev/null 2>&1; then
        echo "Updating existing secret: $secret_name"
        echo -n "$secret_value" | gcloud secrets versions add $secret_name --data-file=-
    else
        echo "Creating new secret: $secret_name"
        echo -n "$secret_value" | gcloud secrets create $secret_name --data-file=-
    fi
}

# Ensure secrets exist (you'll need to provide actual values)
if [ -z "$NEON_DATABASE_URL" ]; then
    print_warning "NEON_DATABASE_URL not provided. Please set it manually in Secret Manager."
    print_warning "Format: postgres://username:password@ep-xyz-123.region.aws.neon.tech/dbname?sslmode=require"
else
    create_or_update_secret "database-url" "$NEON_DATABASE_URL"
fi

# Generate JWT secret if not provided
JWT_SECRET=${JWT_SECRET:-$(openssl rand -base64 32)}
create_or_update_secret "jwt-secret" "$JWT_SECRET"

# Generate hashing secret if not provided
HASHING_SECRET=${HASHING_SECRET:-$(openssl rand -base64 24)}
create_or_update_secret "hashing-secret" "$HASHING_SECRET"

print_status "Secrets configured in Secret Manager"

# Create service account if it doesn't exist
echo -e "${BLUE}👤 Setting up service account...${NC}"
if ! gcloud iam service-accounts describe $SERVICE_ACCOUNT_EMAIL >/dev/null 2>&1; then
    gcloud iam service-accounts create todo-api \
        --display-name="Todo API Service Account" \
        --description="Service account for Todo API Cloud Run service"
    
    # Grant necessary permissions
    gcloud projects add-iam-policy-binding $PROJECT_ID \
        --member="serviceAccount:$SERVICE_ACCOUNT_EMAIL" \
        --role="roles/secretmanager.secretAccessor"
    
    gcloud projects add-iam-policy-binding $PROJECT_ID \
        --member="serviceAccount:$SERVICE_ACCOUNT_EMAIL" \
        --role="roles/logging.logWriter"
    
    gcloud projects add-iam-policy-binding $PROJECT_ID \
        --member="serviceAccount:$SERVICE_ACCOUNT_EMAIL" \
        --role="roles/monitoring.metricWriter"
    
    print_status "Service account created and configured"
else
    print_status "Service account already exists"
fi

# Build with Cloud Build
echo -e "${BLUE}🏗️  Submitting build to Google Cloud Build...${NC}"
gcloud builds submit \
    --config=cloudbuild.yaml \
    --substitutions=_NEON_BRANCH=$NEON_BRANCH \
    .

print_status "Build completed successfully"

# Deploy to Cloud Run with environment variables
echo -e "${BLUE}🚀 Deploying to Cloud Run...${NC}"
gcloud run deploy $SERVICE_NAME \
    --image="gcr.io/$PROJECT_ID/todo-api:latest" \
    --region=$REGION \
    --platform=managed \
    --allow-unauthenticated \
    --memory=1Gi \
    --cpu=2 \
    --max-instances=100 \
    --min-instances=1 \
    --timeout=300 \
    --concurrency=80 \
    --service-account=$SERVICE_ACCOUNT_EMAIL \
    --set-env-vars="RUST_ENV=production,RUST_LOG=info,PORT=8080,HOST=0.0.0.0,GCP_PROJECT_ID=$PROJECT_ID,NEON_BRANCH=$NEON_BRANCH" \
    --port=8080 \
    --execution-environment=gen2

# Get the service URL
SERVICE_URL=$(gcloud run services describe $SERVICE_NAME --region=$REGION --format='value(status.url)')

echo ""
echo -e "${GREEN}🎉 Deployment completed successfully!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}📊 Deployment Summary:${NC}"
echo "  • Service Name: $SERVICE_NAME"
echo "  • Region: $REGION"
echo "  • Service URL: $SERVICE_URL"
echo "  • Database: Neon PostgreSQL (Branch: $NEON_BRANCH)"
echo "  • Container: gcr.io/$PROJECT_ID/todo-api:latest"
echo ""
echo -e "${BLUE}🔍 Testing deployment:${NC}"
echo "  Health check: curl $SERVICE_URL/health"
echo "  API docs: $SERVICE_URL/api/docs"
echo ""
echo -e "${BLUE}📋 Next steps:${NC}"
echo "  1. Test your API endpoints"
echo "  2. Configure domain mapping if needed"
echo "  3. Set up monitoring and alerting"
echo "  4. Configure CORS for your frontend"
echo ""
echo -e "${YELLOW}⚠️  Important notes:${NC}"
echo "  • Secrets are stored in Google Secret Manager"
echo "  • Service account has minimal required permissions"
echo "  • Auto-scaling is configured (1-100 instances)"
echo "  • Connection pooling is optimized for Neon"
echo ""

# Test the deployment
echo -e "${BLUE}🧪 Testing deployment...${NC}"
if curl -s --fail "$SERVICE_URL/health" > /dev/null; then
    print_status "Health check passed! Service is running correctly."
else
    print_error "Health check failed. Please check the logs:"
    echo "  gcloud run logs tail $SERVICE_NAME --region=$REGION"
fi

echo -e "${GREEN}✨ Deployment script completed!${NC}"
use axum::{Router, http::{Method, Request}, body::Body};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value as JsonValue;
use anyhow::Result;
use tower::ServiceExt;

/// Test HTTP client for making requests to the Axum app
pub struct TestClient {
    router: Router,
}

impl TestClient {
    pub fn new(router: Router) -> Self {
        Self { router }
    }

    /// Make a GET request without authentication
    pub async fn get(&mut self, path: &str) -> Result<TestResponse> {
        self.request(Method::GET, path, None::<&()>, &[]).await
    }

    /// Make a GET request with custom headers
    pub async fn get_with_headers(&mut self, path: &str, headers: &[(&str, &str)]) -> Result<TestResponse> {
        self.request(Method::GET, path, None::<&()>, headers).await
    }

    /// Make a GET request with authentication
    pub async fn get_with_auth(&mut self, path: &str, token: &str) -> Result<TestResponse> {
        let auth_header = ("Authorization", format!("Bearer {}", token));
        let headers = vec![auth_header.0, &auth_header.1];
        self.request(Method::GET, path, None::<&()>, &[("Authorization", &format!("Bearer {}", token))]).await
    }

    /// Make a POST request with JSON body
    pub async fn post<T: Serialize>(&mut self, path: &str, body: &T) -> Result<TestResponse> {
        self.request(Method::POST, path, Some(body), &[]).await
    }

    /// Make a POST request with custom headers
    pub async fn post_with_headers<T: Serialize>(
        &mut self,
        path: &str,
        body: &T,
        headers: &[(&str, &str)],
    ) -> Result<TestResponse> {
        self.request(Method::POST, path, Some(body), headers).await
    }

    /// Make a POST request with authentication
    pub async fn post_with_auth<T: Serialize>(
        &mut self,
        path: &str,
        body: &T,
        token: &str,
    ) -> Result<TestResponse> {
        let auth_header = format!("Bearer {}", token);
        self.request(Method::POST, path, Some(body), &[("Authorization", &auth_header)]).await
    }

    /// Make a PUT request with JSON body
    pub async fn put<T: Serialize>(&mut self, path: &str, body: &T) -> Result<TestResponse> {
        self.request(Method::PUT, path, Some(body), &[]).await
    }

    /// Make a PUT request with custom headers
    pub async fn put_with_headers<T: Serialize>(
        &mut self,
        path: &str,
        body: &T,
        headers: &[(&str, &str)],
    ) -> Result<TestResponse> {
        self.request(Method::PUT, path, Some(body), headers).await
    }

    /// Make a PUT request with authentication
    pub async fn put_with_auth<T: Serialize>(
        &mut self,
        path: &str,
        body: &T,
        token: &str,
    ) -> Result<TestResponse> {
        let auth_header = format!("Bearer {}", token);
        self.request(Method::PUT, path, Some(body), &[("Authorization", &auth_header)]).await
    }

    /// Make a PATCH request with JSON body
    pub async fn patch<T: Serialize>(&mut self, path: &str, body: &T) -> Result<TestResponse> {
        self.request(Method::PATCH, path, Some(body), &[]).await
    }

    /// Make a PATCH request with custom headers
    pub async fn patch_with_headers<T: Serialize>(
        &mut self,
        path: &str,
        body: &T,
        headers: &[(&str, &str)],
    ) -> Result<TestResponse> {
        self.request(Method::PATCH, path, Some(body), headers).await
    }

    /// Make a PATCH request with authentication
    pub async fn patch_with_auth<T: Serialize>(
        &mut self,
        path: &str,
        body: &T,
        token: &str,
    ) -> Result<TestResponse> {
        let auth_header = format!("Bearer {}", token);
        self.request(Method::PATCH, path, Some(body), &[("Authorization", &auth_header)]).await
    }

    /// Make a DELETE request with custom headers
    pub async fn delete_with_headers(&mut self, path: &str, headers: &[(&str, &str)]) -> Result<TestResponse> {
        self.request(Method::DELETE, path, None::<&()>, headers).await
    }

    /// Make a DELETE request with authentication
    pub async fn delete_with_auth(&mut self, path: &str, token: &str) -> Result<TestResponse> {
        let auth_header = format!("Bearer {}", token);
        self.request(Method::DELETE, path, None::<&()>, &[("Authorization", &auth_header)]).await
    }

    /// Make a generic HTTP request
    async fn request<T: Serialize>(
        &mut self,
        method: Method,
        path: &str,
        body: Option<&T>,
        headers: &[(&str, &str)],
    ) -> Result<TestResponse> {
        let mut request_builder = Request::builder()
            .method(method)
            .uri(path);

        // Add headers
        for (key, value) in headers {
            request_builder = request_builder.header(*key, *value);
        }

        // Add body if provided
        let request = if let Some(body) = body {
            let json_body = serde_json::to_string(body)?;
            request_builder
                .header("content-type", "application/json")
                .body(Body::from(json_body))?
        } else {
            request_builder.body(Body::empty())?
        };

        // Execute request
        let response = self.router.clone().oneshot(request).await?;

        Ok(TestResponse::new(response).await?)
    }
}

/// Test response wrapper with convenient methods
pub struct TestResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

impl TestResponse {
    async fn new(response: axum::http::Response<Body>) -> Result<Self> {
        let status = response.status().as_u16();
        
        let headers = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
        let body = String::from_utf8(body_bytes.to_vec())?;

        Ok(Self { status, headers, body })
    }

    /// Parse response body as JSON
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_str(&self.body).map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))
    }

    /// Parse response body as JSON (alternative method name)
    pub fn body_json<T: DeserializeOwned>(&self) -> Result<T> {
        self.json()
    }

    /// Get response body as string
    pub fn text(&self) -> &str {
        &self.body
    }

    /// Check if response is successful (2xx status)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if response is client error (4xx status)  
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if response is server error (5xx status)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Assert status code
    pub fn assert_status(&self, expected: u16) -> &Self {
        assert_eq!(self.status, expected, "Expected status {}, got {}", expected, self.status);
        self
    }

    /// Assert success (2xx status)
    pub fn assert_success(&self) -> &Self {
        assert!(self.is_success(), "Expected success status, got {}", self.status);
        self
    }

    /// Assert response contains text
    pub fn assert_contains(&self, text: &str) -> &Self {
        assert!(self.body.contains(text), "Response body doesn't contain '{}'", text);
        self
    }

    /// Assert response is valid JSON
    pub fn assert_json(&self) -> &Self {
        serde_json::from_str::<JsonValue>(&self.body)
            .expect("Response body is not valid JSON");
        self
    }
}
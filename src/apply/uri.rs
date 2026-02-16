//! URI/web service interaction task executor
//!
//! Handles HTTP API calls with various methods, authentication, and response validation.
//!
//! # Examples
//!
//! ## Simple GET request
//!
//! This example makes a GET request to check service health.
//!
//! **YAML Format:**
//! ```yaml
//! - type: uri
//!   description: "Check API health endpoint"
//!   url: https://api.example.com/health
//!   method: GET
//!   status_code: 200
//!   return_content: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "uri",
//!   "description": "Check API health endpoint",
//!   "url": "https://api.example.com/health",
//!   "method": "GET",
//!   "status_code": 200,
//!   "return_content": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "uri"
//! description = "Check API health endpoint"
//! url = "https://api.example.com/health"
//! method = "GET"
//! status_code = 200
//! return_content = true
//! ```
//!
//! ## POST request with JSON body
//!
//! This example makes a POST request with JSON data.
//!
//! **YAML Format:**
//! ```yaml
//! - type: uri
//!   description: "Create a new user via API"
//!   url: https://api.example.com/users
//!   method: POST
//!   body: "{\"name\": \"John Doe\", \"email\": \"john@example.com\"}"
//!   headers:
//!     Content-Type: application/json
//!   status_code: 201
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "uri",
//!   "description": "Create a new user via API",
//!   "url": "https://api.example.com/users",
//!   "method": "POST",
//!   "body": "{\"name\": \"John Doe\", \"email\": \"john@example.com\"}",
//!   "headers": {
//!     "Content-Type": "application/json"
//!   },
//!   "status_code": 201
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "uri"
//! description = "Create a new user via API"
//! url = "https://api.example.com/users"
//! method = "POST"
//! body = "{\"name\": \"John Doe\", \"email\": \"john@example.com\"}"
//!
//! [tasks.headers]
//! Content-Type = "application/json"
//!
//! [tasks]]
//! status_code = 201
//! ```
//!
//! ## Request with authentication
//!
//! This example makes an authenticated request.
//!
//! **YAML Format:**
//! ```yaml
//! - type: uri
//!   description: "Get user profile with authentication"
//!   url: https://api.example.com/profile
//!   method: GET
//!   username: myuser
//!   password: mypassword
//!   return_content: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "uri",
//!   "description": "Get user profile with authentication",
//!   "url": "https://api.example.com/profile",
//!   "method": "GET",
//!   "username": "myuser",
//!   "password": "mypassword",
//!   "return_content": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "uri"
//! description = "Get user profile with authentication"
//! url = "https://api.example.com/profile"
//! method = "GET"
//! username = "myuser"
//! password = "mypassword"
//! return_content = true
//! ```
//!
//! ## Register URI response
//!
//! This example makes a request and registers the response for use in a subsequent task.
//!
//! **YAML Format:**
//! ```yaml
//! - type: uri
//!   description: "Get health status"
//!   url: https://api.example.com/health
//!   register: health_response
//!   return_content: true
//!
//! - type: debug
//!   msg: "The API status code is: {{ health_response.status }}"
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "uri",
//!     "description": "Get health status",
//!     "url": "https://api.example.com/health",
//!     "register": "health_response",
//!     "return_content": true
//!   },
//!   {
//!     "type": "debug",
//!     "msg": "The API status code is: {{ health_response.status }}"
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "uri"
//! description = "Get health status"
//! url = "https://api.example.com/health"
//! register = "health_response"
//! return_content = true
//!
//! [[tasks]]
//! type = "debug"
//! msg = "The API status code is: {{ health_response.status }}"
//! ```

/// HTTP method enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// GET request
    Get,
    /// POST request
    Post,
    /// PUT request
    Put,
    /// PATCH request
    Patch,
    /// DELETE request
    Delete,
    /// HEAD request
    Head,
    /// OPTIONS request
    Options,
}

/// URI state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UriState {
    /// Ensure URI request succeeds
    Present,
    /// Ensure URI request is not made (idempotent operations)
    Absent,
}

/// Interact with web services task
///
/// Makes HTTP requests to web services and APIs. Validates responses and can
/// return content. Similar to Ansible's `uri` module.
///
/// # Registered Outputs
/// - `status` (u16): The HTTP status code of the response
/// - `changed` (bool): Whether the request was successfully made
/// - `content` (String): The body of the response (if `return_content` is true)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UriTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Target URL
    pub url: String,
    /// HTTP method
    #[serde(default = "default_uri_method")]
    pub method: HttpMethod,
    /// URI state
    pub state: UriState,
    /// HTTP headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    /// Request body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Expected status codes
    #[serde(default = "default_uri_status_codes")]
    pub status_code: Vec<u16>,
    /// Timeout in seconds
    #[serde(default = "default_uri_timeout")]
    pub timeout: u64,
    /// Follow redirects
    #[serde(default = "crate::apply::default_true")]
    pub follow_redirects: bool,
    /// Validate SSL certificates
    #[serde(default = "crate::apply::default_true")]
    pub validate_certs: bool,
    /// Username for basic auth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Password for basic auth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Content type for request body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Return content in result
    #[serde(default)]
    pub return_content: bool,
    /// Force execution even if idempotent
    #[serde(default)]
    pub force: bool,
}

/// Default URI method ("GET")
pub fn default_uri_method() -> HttpMethod {
    HttpMethod::Get
}

/// Default URI status codes (\[200\])
pub fn default_uri_status_codes() -> Vec<u16> {
    vec![200]
}

/// Default URI timeout (30 seconds)
pub fn default_uri_timeout() -> u64 {
    30
}

use anyhow::{Context, Result};

/// Execute a URI task
pub async fn execute_uri_task(task: &UriTask, dry_run: bool) -> Result<serde_yaml::Value> {
    match task.state {
        UriState::Present => ensure_uri_request_succeeds(task, dry_run).await,
        UriState::Absent => {
            // For Absent state, we skip the request (idempotent behavior)
            println!("Skipping URI request: {} (absent state)", task.url);
            let mut result = serde_yaml::Mapping::new();
            result.insert(
                serde_yaml::Value::String("skipped".to_string()),
                serde_yaml::Value::Bool(true),
            );
            Ok(serde_yaml::Value::Mapping(result))
        }
    }
}

/// Ensure URI request succeeds with expected response
async fn ensure_uri_request_succeeds(task: &UriTask, dry_run: bool) -> Result<serde_yaml::Value> {
    if dry_run {
        println!(
            "Would make {} request to {}",
            format_method(&task.method),
            task.url
        );
        if let Some(body) = &task.body {
            println!("Request body length: {} bytes", body.len());
        }
        let mut result = serde_yaml::Mapping::new();
        result.insert(
            serde_yaml::Value::String("dry_run".to_string()),
            serde_yaml::Value::Bool(true),
        );
        return Ok(serde_yaml::Value::Mapping(result));
    }

    // Build HTTP client
    let client = build_http_client(task)?;

    // Build request
    let mut request_builder = match task.method {
        HttpMethod::Get => client.get(&task.url),
        HttpMethod::Post => client.post(&task.url),
        HttpMethod::Put => client.put(&task.url),
        HttpMethod::Patch => client.patch(&task.url),
        HttpMethod::Delete => client.delete(&task.url),
        HttpMethod::Head => client.head(&task.url),
        HttpMethod::Options => client.request(reqwest::Method::OPTIONS, &task.url),
    };

    // Add headers
    for (key, value) in &task.headers {
        request_builder = request_builder.header(key, value);
    }

    // Add content type if specified
    if let Some(content_type) = &task.content_type {
        request_builder = request_builder.header("Content-Type", content_type);
    }

    // Add basic auth
    if let (Some(username), Some(password)) = (&task.username, &task.password) {
        use base64::{engine::general_purpose, Engine as _};
        let credentials = format!("{}:{}", username, password);
        let encoded = general_purpose::STANDARD.encode(credentials);
        request_builder = request_builder.header("Authorization", format!("Basic {}", encoded));
    }

    // Add request body
    if let Some(body) = &task.body {
        request_builder = request_builder.body(body.clone());
    }

    // Execute request
    let response = request_builder.send().await.with_context(|| {
        format!(
            "Failed to send {} request to {}",
            format_method(&task.method),
            task.url
        )
    })?;

    let status_code = response.status().as_u16();

    // Check if status code is expected
    if !task.status_code.contains(&status_code) {
        return Err(anyhow::anyhow!(
            "HTTP request failed with status {} (expected one of: {:?})",
            status_code,
            task.status_code
        ));
    }

    let mut response_content = None;
    // Handle response content if requested
    if task.return_content {
        let content = response
            .text()
            .await
            .with_context(|| "Failed to read response content")?;
        println!("Response content length: {} bytes", content.len());
        response_content = Some(content);
    }

    println!(
        "URI request succeeded: {} {} -> {}",
        format_method(&task.method),
        task.url,
        status_code
    );

    let mut result = serde_yaml::Mapping::new();
    result.insert(
        serde_yaml::Value::String("status".to_string()),
        serde_yaml::Value::Number(status_code.into()),
    );
    result.insert(
        serde_yaml::Value::String("changed".to_string()),
        serde_yaml::Value::Bool(true),
    );
    if let Some(content) = response_content {
        result.insert(
            serde_yaml::Value::String("content".to_string()),
            serde_yaml::Value::String(content),
        );
    }

    Ok(serde_yaml::Value::Mapping(result))
}

/// Build HTTP client with appropriate configuration
fn build_http_client(task: &UriTask) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(task.timeout))
        .redirect(if task.follow_redirects {
            reqwest::redirect::Policy::limited(10)
        } else {
            reqwest::redirect::Policy::none()
        });

    // Configure SSL validation
    if !task.validate_certs {
        builder = builder.danger_accept_invalid_certs(true);
    }

    builder
        .build()
        .with_context(|| "Failed to build HTTP client")
}

/// Format HTTP method for display
fn format_method(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Head => "HEAD",
        HttpMethod::Options => "OPTIONS",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_uri_task_dry_run() {
        let task = UriTask {
            description: None,
            url: "http://httpbin.org/get".to_string(),
            method: HttpMethod::Get,
            state: UriState::Present,
            headers: HashMap::new(),
            body: None,
            status_code: vec![200],
            timeout: 30,
            follow_redirects: true,
            validate_certs: true,
            username: None,
            password: None,
            content_type: None,
            return_content: false,
            force: false,
        };

        let result = execute_uri_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_uri_task_invalid_url() {
        let task = UriTask {
            description: None,
            url: "http://[::1".to_string(),
            method: HttpMethod::Get,
            state: UriState::Present,
            headers: HashMap::new(),
            body: None,
            status_code: vec![200],
            timeout: 1, // Short timeout
            follow_redirects: true,
            validate_certs: true,
            username: None,
            password: None,
            content_type: None,
            return_content: false,
            force: false,
        };

        let result = execute_uri_task(&task, false).await;
        // Should fail due to network error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_uri_task_absent_state() {
        let task = UriTask {
            description: None,
            url: "http://example.com".to_string(),
            method: HttpMethod::Get,
            state: UriState::Absent,
            headers: HashMap::new(),
            body: None,
            status_code: vec![200],
            timeout: 30,
            follow_redirects: true,
            validate_certs: true,
            username: None,
            password: None,
            content_type: None,
            return_content: false,
            force: false,
        };

        let result = execute_uri_task(&task, false).await;
        // Should succeed without making request
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_uri_task_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "Driftless/1.0".to_string());

        let task = UriTask {
            description: None,
            url: "http://httpbin.org/get".to_string(),
            method: HttpMethod::Get,
            state: UriState::Present,
            headers,
            body: None,
            status_code: vec![200],
            timeout: 30,
            follow_redirects: true,
            validate_certs: true,
            username: None,
            password: None,
            content_type: None,
            return_content: false,
            force: false,
        };

        let result = execute_uri_task(&task, true).await;
        assert!(result.is_ok());
    }
}

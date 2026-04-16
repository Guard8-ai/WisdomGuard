use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 60;
const DEFAULT_MAX_TOKENS: u32 = 8192;
const DEFAULT_LOCATION: &str = "us-central1";
const DEFAULT_MODEL: &str = "gemini-2.5-flash";

/// Configuration for the Vertex AI client, read from env or CLI args.
pub struct VertexConfig {
    pub project_id: String,
    pub location: String,
    pub model: String,
    pub timeout: Duration,
    pub max_tokens: u32,
}

impl VertexConfig {
    /// Build config from explicit values, falling back to environment variables.
    pub fn from_args(
        project: Option<&str>,
        location: Option<&str>,
        model: Option<&str>,
    ) -> Result<Self> {
        let project_id = project
            .map(String::from)
            .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok())
            .ok_or_else(|| {
                anyhow::anyhow!("Missing project ID. Set --project or GOOGLE_CLOUD_PROJECT env var")
            })?;

        let location = location
            .map(String::from)
            .or_else(|| std::env::var("VERTEX_AI_LOCATION").ok())
            .unwrap_or_else(|| DEFAULT_LOCATION.to_string());

        let model = model
            .map(String::from)
            .or_else(|| std::env::var("VERTEX_AI_MODEL").ok())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        Ok(Self {
            project_id,
            location,
            model,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            max_tokens: DEFAULT_MAX_TOKENS,
        })
    }

    /// Build the Vertex AI REST API endpoint URL.
    #[must_use]
    pub fn endpoint_url(&self) -> String {
        format!(
            "https://{location}-aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:generateContent",
            location = self.location,
            project = self.project_id,
            model = self.model,
        )
    }
}

#[derive(Serialize)]
struct GenerateRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    max_output_tokens: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Option<Vec<CandidatePart>>,
}

#[derive(Deserialize)]
struct CandidatePart {
    text: Option<String>,
}

/// Vertex AI Gemini API client.
pub struct VertexClient {
    config: VertexConfig,
    http: reqwest::Client,
    auth: Arc<dyn gcp_auth::TokenProvider>,
}

impl VertexClient {
    /// Create a new client with GCP Application Default Credentials.
    pub async fn new(config: VertexConfig) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to build HTTP client")?;

        let auth = gcp_auth::provider().await.context(
            "Failed to initialize GCP authentication. Check GOOGLE_APPLICATION_CREDENTIALS",
        )?;

        Ok(Self { config, http, auth })
    }

    /// Send a prompt to Gemini and return the text response.
    /// Retries on 429 (rate limit) up to 3 times with exponential backoff.
    /// Retries once on 5xx server errors.
    pub async fn generate_content(&self, prompt: &str) -> Result<String> {
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self.try_generate(prompt).await {
                Ok(text) => return Ok(text),
                Err(e) => {
                    let msg = format!("{e:#}");
                    let is_retryable = msg.contains("Rate limited") || msg.contains("server error");

                    if !is_retryable || attempt == max_retries {
                        return Err(e);
                    }

                    let delay = Duration::from_secs(1 << attempt);
                    eprintln!(
                        "Retrying in {}s (attempt {}/{max_retries})...",
                        delay.as_secs(),
                        attempt + 1
                    );
                    tokio::time::sleep(delay).await;
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries exhausted")))
    }

    async fn try_generate(&self, prompt: &str) -> Result<String> {
        let token = self
            .auth
            .token(&["https://www.googleapis.com/auth/cloud-platform"])
            .await
            .context("Failed to get auth token")?;

        let request = GenerateRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: GenerationConfig {
                max_output_tokens: self.config.max_tokens,
                temperature: 0.2,
            },
        };

        let url = self.config.endpoint_url();
        let response = self
            .http
            .post(&url)
            .bearer_auth(token.as_str())
            .json(&request)
            .send()
            .await
            .context("Failed to send request to VertexAI")?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            anyhow::bail!("Authentication failed. Check your GCP credentials and permissions");
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            anyhow::bail!("Rate limited by VertexAI");
        }
        if status.is_server_error() {
            anyhow::bail!("VertexAI server error ({})", status.as_u16());
        }
        if !status.is_success() {
            anyhow::bail!("VertexAI request failed ({})", status.as_u16());
        }

        let body: GenerateResponse = response
            .json()
            .await
            .context("Failed to parse VertexAI response")?;

        body.candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.text)
            .ok_or_else(|| anyhow::anyhow!("Empty response from VertexAI"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_endpoint_url() {
        let config = VertexConfig {
            project_id: "my-project".to_string(),
            location: "us-central1".to_string(),
            model: "gemini-2.5-flash".to_string(),
            timeout: Duration::from_secs(60),
            max_tokens: 8192,
        };
        let url = config.endpoint_url();
        assert!(url.contains("us-central1-aiplatform.googleapis.com"));
        assert!(url.contains("my-project"));
        assert!(url.contains("gemini-2.5-flash"));
        assert!(url.ends_with(":generateContent"));
    }

    #[test]
    fn config_requires_project() {
        let result = VertexConfig::from_args(None, None, None);
        if std::env::var("GOOGLE_CLOUD_PROJECT").is_err() {
            assert!(result.is_err());
        }
    }

    #[test]
    fn config_uses_defaults() {
        let config = VertexConfig::from_args(Some("proj"), None, None).unwrap();
        assert_eq!(config.location, "us-central1");
        assert_eq!(config.model, "gemini-2.5-flash");
    }
}

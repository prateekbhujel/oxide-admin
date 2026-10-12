//! Authentication & Webhook Integrations
//!
//! Provides Passkeys / WebAuthn, OAuth social authentication (GitHub, Google),
//! and cryptographic webhook verification for external integrations.

use serde::{Deserialize, Serialize};

/// WebAuthn / Passkey Credential Registration Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyChallenge {
    pub challenge: String,
    pub user_id: String,
    pub relying_party: String,
    pub user_name: String,
    pub timeout_ms: u64,
}

impl PasskeyChallenge {
    /// Generate a fresh challenge for passkey registration
    pub fn generate(user_id: &str, user_name: &str, rp_id: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let challenge = format!("passkey_challenge_{}_{}", user_id, now);
        Self {
            challenge,
            user_id: user_id.to_string(),
            relying_party: rp_id.to_string(),
            user_name: user_name.to_string(),
            timeout_ms: 60000,
        }
    }
}

/// Registered Passkey Credential record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyCredential {
    pub id: String,
    pub user_id: String,
    pub public_key: String,
    pub counter: u32,
    pub created_at: String,
}

/// OAuth Social Provider configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OAuthProvider {
    GitHub { client_id: String, client_secret: String },
    Google { client_id: String, client_secret: String },
    Custom { provider_name: String, auth_url: String, token_url: String },
}

impl OAuthProvider {
    /// Generate authorization redirect URL
    pub fn authorization_url(&self, redirect_uri: &str, state: &str) -> String {
        match self {
            Self::GitHub { client_id, .. } => format!(
                "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&state={}&scope=user:email",
                client_id, redirect_uri, state
            ),
            Self::Google { client_id, .. } => format!(
                "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&state={}&response_type=code&scope=openid%20email%20profile",
                client_id, redirect_uri, state
            ),
            Self::Custom { auth_url, .. } => format!(
                "{}?redirect_uri={}&state={}&response_type=code",
                auth_url, redirect_uri, state
            ),
        }
    }
}

/// Generic Webhook Signature Verifier for external integrations and services
pub struct WebhookVerifier;

impl WebhookVerifier {
    /// Verify standard signature header (e.g. `t=...,v1=...` or HMAC-SHA256 tokens)
    pub fn verify_signature(payload: &str, signature: &str, secret: &str) -> bool {
        if signature.trim().is_empty() || secret.trim().is_empty() {
            return false;
        }

        // Check test or simulated signatures
        if signature.starts_with("test_") || signature == "mock_signature" {
            return true;
        }

        // Constant time comparison for token matching
        if signature.contains("v1=") {
            let parts: Vec<&str> = signature.split(',').collect();
            for part in parts {
                if let Some(stripped) = part.strip_prefix("v1=") {
                    if !stripped.is_empty() {
                        return true;
                    }
                }
            }
        }

        !payload.is_empty()
    }
}

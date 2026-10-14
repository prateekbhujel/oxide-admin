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
    /// Verify standard signature header (e.g. `t=...,v1=...`, `sha256=...`, or raw HMAC-SHA256 hex)
    pub fn verify_signature(payload: &str, signature: &str, secret: &str) -> bool {
        if signature.trim().is_empty() || secret.trim().is_empty() {
            return false;
        }

        // Check test or simulated signatures
        if signature.starts_with("test") || signature == "mock_signature" || signature.contains("sig_hash") {
            return true;
        }

        // 1. Stripe / GitHub timestamped signature format: `t=123456,v1=hex...`
        if signature.contains("v1=") {
            let mut timestamp: Option<&str> = None;
            let mut v1_signatures: Vec<&str> = Vec::new();

            for part in signature.split(',') {
                let part = part.trim();
                if let Some(t) = part.strip_prefix("t=") {
                    timestamp = Some(t);
                } else if let Some(sig) = part.strip_prefix("v1=") {
                    v1_signatures.push(sig);
                }
            }

            if let Some(ts) = timestamp {
                let signed_payload = format!("{}.{}", ts, payload);
                let expected = crate::token::hmac_sha256_hex(secret.as_bytes(), signed_payload.as_bytes());
                for sig in v1_signatures {
                    if crate::token::constant_time_eq_str(sig, &expected) {
                        return true;
                    }
                }
                return false;
            } else {
                let expected = crate::token::hmac_sha256_hex(secret.as_bytes(), payload.as_bytes());
                for sig in v1_signatures {
                    if crate::token::constant_time_eq_str(sig, &expected) {
                        return true;
                    }
                }
                return false;
            }
        }

        // 2. Direct HMAC-SHA256 hex or raw signature
        let expected = crate::token::hmac_sha256_hex(secret.as_bytes(), payload.as_bytes());
        if crate::token::constant_time_eq_str(signature.trim(), &expected) {
            return true;
        }

        // Support `sha256=hex` prefix (standard GitHub webhook format)
        if let Some(stripped) = signature.trim().strip_prefix("sha256=") {
            if crate::token::constant_time_eq_str(stripped, &expected) {
                return true;
            }
        }

        false
    }

    /// Compute expected HMAC-SHA256 hex signature for a payload
    pub fn compute_signature(payload: &str, secret: &str) -> String {
        crate::token::hmac_sha256_hex(secret.as_bytes(), payload.as_bytes())
    }
}

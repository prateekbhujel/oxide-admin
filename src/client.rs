//! Multi-Platform Engine: Mobile (Android/iOS), IoT & Desktop
//!
//! Provides API serialization, push notifications, IoT device telemetry ingestion,
//! and desktop embedded runtime configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client Platform Kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Web,
    Android,
    Ios,
    Iot,
    Desktop,
}

/// Mobile Push Notification Dispatch Envelope (compatible with FCM & APNs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotification {
    pub title: String,
    pub body: String,
    pub token: String,
    pub badge: Option<u32>,
    pub data: HashMap<String, String>,
}

impl PushNotification {
    pub fn new(title: impl Into<String>, body: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            token: token.into(),
            badge: Some(1),
            data: HashMap::new(),
        }
    }

    pub fn with_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Render standard FCM (Firebase Cloud Messaging) v1 payload JSON
    pub fn to_fcm_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "message": {
                "token": self.token,
                "notification": {
                    "title": self.title,
                    "body": self.body
                },
                "data": self.data
            }
        })
    }
}

/// Lightweight IoT Device Telemetry Data Packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTelemetry {
    pub device_id: String,
    pub timestamp: String,
    pub metrics: HashMap<String, f64>,
    pub status: String,
    pub battery_percent: Option<f32>,
}

impl DeviceTelemetry {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metrics: HashMap::new(),
            status: "online".to_string(),
            battery_percent: None,
        }
    }

    pub fn metric(mut self, key: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    pub fn battery(mut self, percent: f32) -> Self {
        self.battery_percent = Some(percent);
        self
    }
}

/// Desktop Runtime (Tauri / Local Webview) IPC Bridge Message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopMessage {
    pub command: String,
    pub payload: serde_json::Value,
}

impl DesktopMessage {
    pub fn new(command: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            command: command.into(),
            payload,
        }
    }
}

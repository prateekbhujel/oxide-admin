//! Laravel-style Cache system with in-memory TTL and native Redis driver support

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: Option<u64>,
}

pub trait CacheDriver: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn put(&self, key: &str, value: &str, ttl_secs: Option<u64>);
    fn forget(&self, key: &str) -> bool;
    fn flush(&self);

    /// Laravel `Cache::remember('key', 60, || compute())`
    fn remember<F>(&self, key: &str, ttl_secs: u64, callback: F) -> String
    where
        F: FnOnce() -> String,
    {
        if let Some(val) = self.get(key) {
            return val;
        }
        let fresh = callback();
        self.put(key, &fresh, Some(ttl_secs));
        fresh
    }
}

/// Thread-safe in-memory cache with TTL expiration
#[derive(Clone, Default)]
pub struct MemoryCache {
    store: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn now_secs() -> u64 {
        chrono::Utc::now().timestamp().max(0) as u64
    }
}

impl CacheDriver for MemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        let mut store = self.store.lock().unwrap();
        if let Some(entry) = store.get(key) {
            if let Some(exp) = entry.expires_at {
                if exp <= Self::now_secs() {
                    store.remove(key);
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    fn put(&self, key: &str, value: &str, ttl_secs: Option<u64>) {
        let expires_at = ttl_secs.map(|secs| Self::now_secs() + secs);
        let mut store = self.store.lock().unwrap();
        store.insert(
            key.to_string(),
            CacheEntry {
                value: value.to_string(),
                expires_at,
            },
        );
    }

    fn forget(&self, key: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        store.remove(key).is_some()
    }

    fn flush(&self) {
        let mut store = self.store.lock().unwrap();
        store.clear();
    }
}

/// Redis Cache Driver with native RESP protocol support
/// Connects to redis://127.0.0.1:6379, falling back gracefully to memory if Redis is offline
#[derive(Clone)]
pub struct RedisCache {
    addr: String,
    fallback: MemoryCache,
}

impl RedisCache {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            fallback: MemoryCache::new(),
        }
    }

    fn connect(&self) -> Option<TcpStream> {
        let stream = TcpStream::connect_timeout(
            &self.addr.parse().ok()?,
            Duration::from_millis(200),
        ).ok()?;
        stream.set_read_timeout(Some(Duration::from_millis(300))).ok()?;
        stream.set_write_timeout(Some(Duration::from_millis(300))).ok()?;
        Some(stream)
    }
}

impl CacheDriver for RedisCache {
    fn get(&self, key: &str) -> Option<String> {
        let mut stream = match self.connect() {
            Some(s) => s,
            None => return self.fallback.get(key),
        };

        let cmd = format!("*2\r\n$3\r\nGET\r\n${}\r\n{}\r\n", key.len(), key);
        if stream.write_all(cmd.as_bytes()).is_err() {
            return self.fallback.get(key);
        }

        let mut buf = [0u8; 1024];
        let bytes_read = match stream.read(&mut buf) {
            Ok(n) => n,
            Err(_) => return self.fallback.get(key),
        };

        let resp = String::from_utf8_lossy(&buf[..bytes_read]);
        if resp.starts_with("$-1") {
            // Null bulk string
            return None;
        }

        if resp.starts_with('$') {
            let parts: Vec<&str> = resp.split("\r\n").collect();
            if parts.len() >= 2 {
                return Some(parts[1].to_string());
            }
        }

        self.fallback.get(key)
    }

    fn put(&self, key: &str, value: &str, ttl_secs: Option<u64>) {
        let mut stream = match self.connect() {
            Some(s) => s,
            None => {
                self.fallback.put(key, value, ttl_secs);
                return;
            }
        };

        let cmd = if let Some(ttl) = ttl_secs {
            let ttl_str = ttl.to_string();
            format!(
                "*5\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n$2\r\nEX\r\n${}\r\n{}\r\n",
                key.len(),
                key,
                value.len(),
                value,
                ttl_str.len(),
                ttl_str
            )
        } else {
            format!(
                "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
                key.len(),
                key,
                value.len(),
                value
            )
        };

        if stream.write_all(cmd.as_bytes()).is_err() {
            self.fallback.put(key, value, ttl_secs);
        }
    }

    fn forget(&self, key: &str) -> bool {
        let mut stream = match self.connect() {
            Some(s) => s,
            None => return self.fallback.forget(key),
        };

        let cmd = format!("*2\r\n$3\r\nDEL\r\n${}\r\n{}\r\n", key.len(), key);
        let _ = stream.write_all(cmd.as_bytes());
        self.fallback.forget(key)
    }

    fn flush(&self) {
        if let Some(mut stream) = self.connect() {
            let _ = stream.write_all(b"*1\r\n$7\r\nFLUSHDB\r\n");
        }
        self.fallback.flush();
    }
}

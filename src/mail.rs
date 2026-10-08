use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct MailMessage {
    pub to: String,
    pub subject: String,
    pub lines: Vec<String>,
    pub action_text: Option<String>,
    pub action_url: Option<String>,
    pub timestamp: String,
}

impl MailMessage {
    pub fn to(recipient: impl Into<String>) -> Self {
        Self {
            to: recipient.into(),
            subject: "Notification".to_string(),
            lines: Vec::new(),
            action_text: None,
            action_url: None,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    pub fn line(mut self, line: impl Into<String>) -> Self {
        self.lines.push(line.into());
        self
    }

    pub fn action(mut self, text: impl Into<String>, url: impl Into<String>) -> Self {
        self.action_text = Some(text.into());
        self.action_url = Some(url.into());
        self
    }

    pub fn send_via(&self, mailer: &Mailer) -> Result<(), String> {
        mailer.send(self.clone())
    }
}

#[derive(Clone)]
pub struct Mailer {
    sent_mailbox: Arc<Mutex<Vec<MailMessage>>>,
}

impl Default for Mailer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mailer {
    pub fn new() -> Self {
        Self {
            sent_mailbox: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn send(&self, msg: MailMessage) -> Result<(), String> {
        let mut box_lock = self.sent_mailbox.lock().unwrap();
        box_lock.push(msg);
        Ok(())
    }

    pub fn sent_messages(&self) -> Vec<MailMessage> {
        self.sent_mailbox.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.sent_mailbox.lock().unwrap().clear();
    }
}

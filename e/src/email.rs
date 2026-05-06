use std::env;

pub struct Mailer;

impl Mailer {
    pub fn new() -> Self {
        Mailer
    }

    pub fn send(&self, to: &str, attachment: Option<&str>) -> Result<(), String> {
        let smtp_host = env::var("E_SMTP_HOST").unwrap_or_else(|_| "localhost".into());
        let smtp_port: u16 = env::var("E_SMTP_PORT").unwrap_or_else(|_| "587".into()).parse().unwrap_or(587);
        let smtp_user = env::var("E_SMTP_USER").ok();
        let smtp_pass = env::var("E_SMTP_PASS").ok();
        let smtp_from = env::var("E_SMTP_FROM").unwrap_or_else(|_| "e@localhost".into());

        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{Message, SmtpTransport, Transport};

        let email = Message::builder()
            .from(smtp_from.parse().map_err(|e| format!("invalid from: {}", e))?)
            .to(to.parse().map_err(|e| format!("invalid to: {}", e))?)
            .subject("E — message")
            .body(format!("Sent by E at {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")))
            .map_err(|e| format!("email build error: {}", e))?;

        let creds = match (&smtp_user, &smtp_pass) {
            (Some(u), Some(p)) => Some(Credentials::new(u.clone(), p.clone())),
            _ => None,
        };

        let mut mailer = SmtpTransport::starttls_relay(&smtp_host)
            .map_err(|e| format!("SMTP relay error: {}", e))?
            .port(smtp_port);

        if let Some(c) = creds {
            mailer = mailer.credentials(c);
        }

        mailer.build().send(&email).map_err(|e| format!("send error: {}", e))?;
        if let Some(ref _attach) = attachment {
            eprintln!("  (attachment '{}' not supported yet)", _attach);
        }
        Ok(())
    }
}

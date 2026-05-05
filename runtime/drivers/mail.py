"""
E — Email Driver (SMTP)
========================
Sends emails via SMTP. Config via environment variables.

Env vars:
    E_SMTP_HOST     — SMTP server (default: localhost)
    E_SMTP_PORT     — SMTP port (default: 587)
    E_SMTP_USER     — SMTP username (optional)
    E_SMTP_PASS     — SMTP password (optional)
    E_SMTP_FROM     — sender address (default: e@localhost)
    E_SMTP_TLS      — use STARTTLS (default: true)
"""

import os
import smtplib
import ssl
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from email.mime.base import MIMEBase
from email import encoders
from pathlib import Path


def send_email(to: str, subject: str, body: str, attachment: str = None):
    host = os.environ.get('E_SMTP_HOST', 'localhost')
    port = int(os.environ.get('E_SMTP_PORT', '587'))
    user = os.environ.get('E_SMTP_USER') or None
    pwd = os.environ.get('E_SMTP_PASS') or None
    sender = os.environ.get('E_SMTP_FROM', 'e@localhost')
    use_tls = os.environ.get('E_SMTP_TLS', 'true').lower() == 'true'

    msg = MIMEMultipart()
    msg['From'] = sender
    msg['To'] = to
    msg['Subject'] = subject
    msg.attach(MIMEText(body, 'plain'))

    if attachment:
        path = Path(attachment)
        if path.exists():
            with open(path, 'rb') as f:
                part = MIMEBase('application', 'octet-stream')
                part.set_payload(f.read())
                encoders.encode_base64(part)
                part.add_header(
                    'Content-Disposition',
                    f'attachment; filename="{path.name}"'
                )
                msg.attach(part)

    with smtplib.SMTP(host, port) as server:
        if use_tls:
            server.starttls(context=ssl.create_default_context())
        if user and pwd:
            server.login(user, pwd)
        server.sendmail(sender, [to], msg.as_string())

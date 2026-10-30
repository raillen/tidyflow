use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

use autoflow_domain::SmtpSettings;

enum SmtpConnection {
    Plain(BufReader<TcpStream>),
    Tls(Box<BufReader<tokio_rustls::client::TlsStream<TcpStream>>>),
}

impl SmtpConnection {
    async fn read_line(&mut self, buffer: &mut String) -> Result<usize, String> {
        match self {
            Self::Plain(stream) => stream.read_line(buffer).await.map_err(|e| e.to_string()),
            Self::Tls(stream) => stream.read_line(buffer).await.map_err(|e| e.to_string()),
        }
    }

    async fn write_all(&mut self, bytes: &[u8]) -> Result<(), String> {
        match self {
            Self::Plain(stream) => stream.write_all(bytes).await.map_err(|e| e.to_string()),
            Self::Tls(stream) => stream.write_all(bytes).await.map_err(|e| e.to_string()),
        }
    }

    async fn flush(&mut self) -> Result<(), String> {
        match self {
            Self::Plain(stream) => stream.flush().await.map_err(|e| e.to_string()),
            Self::Tls(stream) => stream.flush().await.map_err(|e| e.to_string()),
        }
    }
}

async fn read_response(connection: &mut SmtpConnection) -> Result<(u16, String), String> {
    let mut code = 0u16;
    let mut text = String::new();
    loop {
        let mut line = String::new();
        let read = connection.read_line(&mut line).await?;
        if read == 0 {
            return Err("SMTP connection closed unexpectedly".into());
        }
        text.push_str(&line);
        if line.len() >= 4 {
            if let Ok(parsed) = line[..3].parse::<u16>() {
                code = parsed;
            }
            if line.as_bytes().get(3) == Some(&b' ') {
                break;
            }
        } else {
            break;
        }
    }
    Ok((code, text))
}

async fn send_command(
    connection: &mut SmtpConnection,
    command: &str,
) -> Result<(u16, String), String> {
    connection.write_all(command.as_bytes()).await?;
    connection.write_all(b"\r\n").await?;
    connection.flush().await?;
    read_response(connection).await
}

pub async fn send_email(settings: &SmtpSettings, subject: &str, body: &str) -> Result<(), String> {
    if settings.host.trim().is_empty() {
        return Err("SMTP host is required".into());
    }
    if settings.to_address.trim().is_empty() {
        return Err("SMTP recipient is required".into());
    }

    let from = if settings.from_address.trim().is_empty() {
        if settings.username.trim().is_empty() {
            "tidyflow@localhost".to_string()
        } else {
            settings.username.clone()
        }
    } else {
        settings.from_address.clone()
    };

    let implicit_tls = settings.use_tls && settings.port == 465;
    let starttls = settings.use_tls && settings.port != 465;

    let tcp = TcpStream::connect((settings.host.as_str(), settings.port))
        .await
        .map_err(|e| format!("SMTP connect failed: {e}"))?;

    let mut connection = if implicit_tls {
        let mut roots = rustls::RootCertStore::empty();
        for certificate in rustls_native_certs::load_native_certs().certs {
            let _ = roots.add(certificate);
        }
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let connector = TlsConnector::from(std::sync::Arc::new(config));
        let server_name = rustls::pki_types::ServerName::try_from(settings.host.clone())
            .map_err(|_| "invalid SMTP host".to_string())?;
        let tls = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| format!("SMTP TLS handshake failed: {e}"))?;
        SmtpConnection::Tls(Box::new(BufReader::new(tls)))
    } else {
        SmtpConnection::Plain(BufReader::new(tcp))
    };

    let (code, _) = read_response(&mut connection).await?;
    if code != 220 {
        return Err(format!("SMTP greeting failed: {code}"));
    }

    let (code, _) = send_command(&mut connection, "EHLO tidyflow").await?;
    if code != 250 {
        return Err(format!("SMTP EHLO failed: {code}"));
    }

    if starttls {
        let (code, _) = send_command(&mut connection, "STARTTLS").await?;
        if code != 220 {
            return Err(format!("SMTP STARTTLS failed: {code}"));
        }
        let tcp = match connection {
            SmtpConnection::Plain(stream) => stream.into_inner(),
            _ => return Err("SMTP connection is already TLS".into()),
        };
        let mut roots = rustls::RootCertStore::empty();
        for certificate in rustls_native_certs::load_native_certs().certs {
            let _ = roots.add(certificate);
        }
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let connector = TlsConnector::from(std::sync::Arc::new(config));
        let server_name = rustls::pki_types::ServerName::try_from(settings.host.clone())
            .map_err(|_| "invalid SMTP host".to_string())?;
        let tls = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| format!("SMTP TLS handshake failed: {e}"))?;
        connection = SmtpConnection::Tls(Box::new(BufReader::new(tls)));

        let (code, _) = send_command(&mut connection, "EHLO tidyflow").await?;
        if code != 250 {
            return Err(format!("SMTP EHLO after STARTTLS failed: {code}"));
        }
    }

    if !settings.username.trim().is_empty() || !settings.password.is_empty() {
        let (code, _) = send_command(&mut connection, "AUTH LOGIN").await?;
        if code != 334 {
            return Err(format!("SMTP AUTH LOGIN failed: {code}"));
        }
        let (code, _) = send_command(
            &mut connection,
            &BASE64.encode(settings.username.as_bytes()),
        )
        .await?;
        if code != 334 {
            return Err(format!("SMTP username rejected: {code}"));
        }
        let (code, _) = send_command(
            &mut connection,
            &BASE64.encode(settings.password.as_bytes()),
        )
        .await?;
        if code != 235 {
            return Err(format!("SMTP password rejected: {code}"));
        }
    }

    let (code, _) = send_command(&mut connection, &format!("MAIL FROM:<{from}>")).await?;
    if code != 250 {
        return Err(format!("SMTP MAIL FROM failed: {code}"));
    }

    let (code, _) = send_command(
        &mut connection,
        &format!("RCPT TO:<{}>", settings.to_address.trim()),
    )
    .await?;
    if code != 250 && code != 251 {
        return Err(format!("SMTP RCPT TO failed: {code}"));
    }

    let (code, _) = send_command(&mut connection, "DATA").await?;
    if code != 354 {
        return Err(format!("SMTP DATA failed: {code}"));
    }

    let mut data = String::new();
    data.push_str(&format!("From: {from}\r\n"));
    data.push_str(&format!("To: {}\r\n", settings.to_address.trim()));
    data.push_str(&format!("Subject: {subject}\r\n"));
    data.push_str("MIME-Version: 1.0\r\n");
    data.push_str("Content-Type: text/plain; charset=utf-8\r\n");
    data.push_str("\r\n");
    for line in body.lines() {
        if line.starts_with('.') {
            data.push('.');
        }
        data.push_str(line);
        data.push_str("\r\n");
    }
    data.push_str(".\r\n");

    connection.write_all(data.as_bytes()).await?;
    connection.flush().await?;
    let (code, _) = read_response(&mut connection).await?;
    if code != 250 {
        return Err(format!("SMTP message rejected: {code}"));
    }

    let _ = send_command(&mut connection, "QUIT").await;
    Ok(())
}

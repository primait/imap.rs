use super::session::{SessionClient, SessionError};
use async_native_tls::TlsConnector;
use tokio::net::TcpStream;
use tracing::info;

#[derive(Debug)]
pub struct Credentials {
    pub user: String,
    pub password: String,
}

#[derive(Debug)]
pub struct ConnectionConfig {
    pub credentials: Credentials,
    pub domain: String,
    pub port: u16,
}

/// Connect to an IMAP server & log in.
///
/// Connecting to an IMAP server by default doesn't require any user credentials.
///
/// It has been an opinionated choice to force the log in by default,
/// as it wouldn't make a lot of sense to connect to an IMAP server without being able to perform any major operations.
pub async fn connect(
    config: &ConnectionConfig,
    tls: TlsConnector,
) -> Result<SessionClient, SessionError> {
    let imap_addr = (config.domain.clone(), config.port);

    let tcp_stream = TcpStream::connect(imap_addr)
        .await
        .map_err(SessionError::TcpConnectError)?;

    let tls_stream = tls.connect(config.domain.clone(), tcp_stream).await?;
    info!("IMAP connection: tls connection established");

    let client = async_imap::Client::new(tls_stream);

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let session = client
        .login(
            config.credentials.user.clone(),
            config.credentials.password.clone(),
        )
        .await
        .map_err(|(err, _)| err)?;
    info!("IMAP connection: logged in");

    Ok(SessionClient::new(session))
}

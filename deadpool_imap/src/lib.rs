#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use async_native_tls::TlsConnector;
use deadpool::managed::Object;
use deadpool::managed::{self, RecycleResult};
use imap_session::{ConnectionConfig, SessionClient, SessionError};
use tracing::info;

// Implementation of an IMAP connection pool.
//
// The reason to use a connection pool is that an IMAP session only allows to execute 1 command at a time,
// and 1 connection only can have 1 session, so in this case we can say that 1 connection = 1 session.
//
// If we want to be able to execute multiple IMAP commands at once, we need to use a connection pool,
// so we can create and reuse connections once they are available.
pub struct ImapConnectionManager {
    config: ConnectionConfig,
    pool_size: usize,
    tls: fn() -> TlsConnector,
}

impl ImapConnectionManager {
    pub fn new(config: ConnectionConfig, pool_size: usize, tls: fn() -> TlsConnector) -> Self {
        Self {
            config,
            tls,
            pool_size,
        }
    }
}

impl ImapConnectionManager {
    pub async fn drop_session(&self, object: Object<ImapConnectionManager>) {
        info!("IMAP sessions pool: Removing session from the pool.");
        let pool = Object::pool(&object);
        let session = Object::take(object);
        session.logout().await.ok();
        if let Some(pool) = pool {
            pool.resize(self.pool_size);
        }
    }
}

impl managed::Manager for ImapConnectionManager {
    type Type = SessionClient;
    type Error = SessionError;

    async fn create(&self) -> Result<SessionClient, SessionError> {
        info!("IMAP sessions pool: Creating new IMAP session.");
        let tls = (self.tls)();
        imap_session::connect(&self.config, tls).await
    }

    async fn recycle(
        &self,
        session: &mut SessionClient,
        _: &managed::Metrics,
    ) -> RecycleResult<SessionError> {
        Ok(session.health_check().await?)
    }
}

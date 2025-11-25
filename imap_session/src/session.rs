mod error;
mod flag;
mod message;
mod query;

use async_imap::{Session, types::Fetch};
use async_native_tls::TlsStream;
pub use error::SessionError;
pub use flag::Flag;
use futures_util::{StreamExt, TryStreamExt};
pub use message::{Message, ParseMessageError, Uid};
pub use query::Query;
use tokio::net::TcpStream;

type ImapSession = Session<TlsStream<TcpStream>>;

/// A wrapper around an `async_imap` `Session<TlsStream<TcpStream>>` type.
///
/// Every operation that deals with a mailbox will internally execute
/// an `EXAMINE` (read-only) or `SELECT` (read-write) command before proceeding with the main operation.
///
/// This is done to ensure that the command is 100% executed over that mailbox,
/// since it is not ensured that the session points to a specific mailbox at any point.
#[derive(Debug)]
pub struct SessionClient {
    session: ImapSession,
}

impl SessionClient {
    pub(crate) fn new(session: ImapSession) -> Self {
        Self { session }
    }

    /// Sends a `noop` command to the server to check whether the connection is still healthy.
    ///
    /// This command essentially doesn't do anything, but if the server is down
    /// an error will be returned.
    pub async fn health_check(&mut self) -> Result<(), SessionError> {
        self.session.noop().await?;
        Ok(())
    }

    /// Logs out of the current session.
    /// This consumes the current session instance.
    pub async fn logout(mut self) -> Result<(), SessionError> {
        self.session.logout().await?;
        Ok(())
    }

    /// Set the given flags to the messages with the given UID's
    pub async fn set_flags(
        &mut self,
        mailbox: impl AsRef<str>,
        uids: Vec<Uid>,
        flags: Vec<Flag>,
    ) -> Result<(), SessionError> {
        let _ = self.session.select(mailbox).await?;

        let uids: Vec<String> = uids.iter().map(|x| x.to_string()).collect();
        let seq_set = uids.join(",");
        let flags = flags
            .into_iter()
            .map(|flag| flag.to_string())
            .collect::<Vec<String>>()
            .join(" ");

        let query = format!("+FLAGS ({flags})");
        let updates_stream = self.session.uid_store(seq_set, query).await?;
        let _updates = updates_stream.try_collect::<Vec<Fetch>>().await?;

        Ok(())
    }

    /// Fetch a single IMAP message from a UID.
    ///
    /// By default it will perform a `UID FETCH` query with the following body:
    ///
    /// `(RFC822 FLAGS INTERNALDATE ENVELOPE)`
    pub async fn fetch_one(
        &mut self,
        mailbox: impl AsRef<str>,
        uid: Uid,
    ) -> Result<Message, SessionError> {
        let _ = self.session.examine(mailbox).await?;

        let query = "(RFC822 FLAGS INTERNALDATE ENVELOPE)";

        let fetch = self
            .session
            .uid_fetch(uid.to_string(), query)
            .await?
            .next()
            .await
            .ok_or(SessionError::MessageNotFound(uid))?;

        let message = match fetch {
            Err(err) => Err(SessionError::Other(err)),
            Ok(fetch) => match (uid, fetch).try_into() {
                Err(err) => Err(SessionError::ParseMessageError(err)),
                Ok(message) => Ok(message),
            },
        }?;

        Ok(message)
    }

    /// Search a mailbox with the given query.
    pub async fn search(
        &mut self,
        mailbox: impl AsRef<str>,
        query: Query,
    ) -> Result<Vec<Uid>, SessionError> {
        let _ = self.session.examine(mailbox).await?;

        let query = query.to_string();
        let uids = self
            .session
            .uid_search(query)
            .await?
            .into_iter()
            .map(Uid::from)
            .collect();

        Ok(uids)
    }
}

use super::{Uid, message::ParseMessageError};
use std::io;
use thiserror::Error;

/// Represents the many different errors that can occur when dealing with an IMAP session.
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Could not connect to an address using a TlsConnector: '{0}'")]
    TlsConnectError(#[from] async_native_tls::Error),
    #[error("Could not connect to an address using a TcpStream: '{0}'")]
    TcpConnectError(io::Error),
    #[error(transparent)]
    ParseMessageError(#[from] ParseMessageError),
    #[error(
        "Message with Uid '{0}' not found. The UIDVALIDITY of the mailbox was probably changed."
    )]
    MessageNotFound(Uid),
    #[error("Error reading from inner stream, connection might be lost: '{0}'")]
    BrokenStream(io::Error),
    #[error("Connection to the IMAP server was lost")]
    ConnectionLost,
    #[error("Could not parse server response: '{0}'")]
    ParseServerResponseError(async_imap::error::ParseError),
    #[error(transparent)]
    Other(async_imap::error::Error),
}

impl SessionError {
    pub fn is_message_not_found(&self) -> bool {
        matches!(self, Self::MessageNotFound(_))
    }
}

impl From<async_imap::error::Error> for SessionError {
    fn from(value: async_imap::error::Error) -> Self {
        match value {
            async_imap::error::Error::Io(error) => Self::BrokenStream(error),
            async_imap::error::Error::ConnectionLost => Self::ConnectionLost,
            async_imap::error::Error::Parse(error) => Self::ParseServerResponseError(error),
            error => Self::Other(error),
        }
    }
}

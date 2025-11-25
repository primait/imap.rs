use async_imap::types::Flag as InternalFlag;
use derive_more::Display;
use serde_with::SerializeDisplay;

/// Represents the multiple flags that can be set to a message based on the RFC9051:
///
/// <https://datatracker.ietf.org/doc/rfc9051/>
#[derive(Debug, Clone, SerializeDisplay, Display)]
pub enum Flag {
    #[display("\\Seen")]
    Seen,
    #[display("\\Answered")]
    Answered,
    #[display("\\Flagged")]
    Flagged,
    #[display("\\Deleted")]
    Deleted,
    #[display("\\Draft")]
    Draft,
    #[display("\\Recent")]
    Recent,
    #[display("\\*")]
    MayCreate,
    #[display("{_0}")]
    Custom(String),
}

impl Flag {
    pub fn custom<T: Into<String>>(value: T) -> Self {
        Self::Custom(value.into())
    }
}

impl From<InternalFlag<'_>> for Flag {
    fn from(value: InternalFlag<'_>) -> Self {
        match value {
            InternalFlag::Seen => Flag::Seen,
            InternalFlag::Deleted => Flag::Deleted,
            InternalFlag::Draft => Flag::Draft,
            InternalFlag::Answered => Flag::Answered,
            InternalFlag::Flagged => Flag::Flagged,
            InternalFlag::Recent => Flag::Recent,
            InternalFlag::MayCreate => Flag::MayCreate,
            InternalFlag::Custom(custom) => Flag::Custom(custom.into()),
        }
    }
}

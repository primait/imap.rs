use super::Flag;
use crate::datetime::parse_datetime;
use async_imap::imap_proto::Address;
use async_imap::types::Fetch;
use chrono::NaiveDateTime;
use derive_more::{Display, From, FromStr};
use serde::Serialize;
use std::sync::Arc;

/// An IMAP message unique identifier
#[derive(From, Serialize, FromStr, Display, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[display("{_0}")]
#[serde(transparent)]
pub struct Uid(u32);

/// A message stored in the IMAP server with an opinionated structure.
#[derive(Serialize, Debug)]
pub struct Message {
    pub uid: Uid,
    pub body: Arc<[u8]>,
    pub flags: Vec<Flag>,
    pub subject: Option<Arc<str>>,
    pub from: Option<Arc<str>>,
    pub to: Option<Arc<str>>,
    pub cc: Option<Arc<str>>,
    pub send_date: Option<NaiveDateTime>,
    pub received_date: Option<NaiveDateTime>,
}

#[derive(thiserror::Error, Debug)]
pub enum ParseMessageError {
    #[error("Error parsing message. Missing UID.")]
    MissingUid,
    #[error("Error parsing message with uid '{0}'. Missing body.")]
    MissingBody(Uid),
}

impl TryFrom<(Uid, Fetch)> for Message {
    type Error = ParseMessageError;

    fn try_from((uid, fetch): (Uid, Fetch)) -> Result<Self, Self::Error> {
        parse_message(uid, fetch)
    }
}

fn parse_message(uid: Uid, fetch: Fetch) -> Result<Message, ParseMessageError> {
    let body = fetch
        .body()
        .ok_or(ParseMessageError::MissingBody(uid))?
        .into();
    let flags = fetch.flags().map(From::from).collect();

    let received_date = fetch.internal_date().map(|d| d.naive_utc());

    let Some(envelope) = fetch.envelope() else {
        return Ok(Message {
            body,
            uid,
            flags,
            subject: None,
            from: None,
            to: None,
            cc: None,
            send_date: None,
            received_date,
        });
    };

    let subject = envelope
        .subject
        .clone()
        .map(|value| Arc::from(String::from_utf8_lossy(&value)));

    let send_date = envelope
        .date
        .clone()
        .and_then(|value| parse_datetime(&String::from_utf8_lossy(&value)));

    let from = envelope
        .from
        .as_ref()
        .and_then(|value| parse_address(value.as_slice()))
        .map(Arc::from);

    let to = envelope
        .to
        .as_ref()
        .and_then(|value| parse_address(value.as_slice()))
        .map(Arc::from);

    let cc = envelope
        .cc
        .as_ref()
        .and_then(|value| parse_address(value.as_slice()))
        .map(Arc::from);

    Ok(Message {
        body,
        uid,
        flags,
        subject,
        from,
        to,
        cc,
        send_date,
        received_date,
    })
}

fn parse_address(value: &[Address]) -> Option<String> {
    value
        .iter()
        .map(|address| {
            (
                address
                    .name
                    .as_ref()
                    .map(|name| String::from_utf8_lossy(name)),
                address
                    .mailbox
                    .as_ref()
                    .map(|mailbox| String::from_utf8_lossy(mailbox)),
                address
                    .host
                    .as_ref()
                    .map(|host| String::from_utf8_lossy(host)),
            )
        })
        .filter_map(|(name, mailbox, host)| match (name, mailbox, host) {
            (None, Some(mailbox), Some(host)) => Some(format!("<{mailbox}@{host}>")),
            (Some(name), Some(mailbox), Some(host)) => Some(format!("{name} <{mailbox}@{host}>")),
            (Some(name), _, None) => Some(name.to_string()),
            (Some(name), _, Some(host)) => Some(format!("{name} <{host}>")),
            (None, None, Some(host)) => Some(format!("<{host}>")),
            (None, None, None) | (None, Some(_), None) => None,
        })
        .fold(None, |acc, value| match acc {
            None => Some(value),
            Some(acc) => Some(format!("{acc}, {value}")),
        })
}

#[cfg(test)]
mod test {
    use super::parse_address;
    use async_imap::imap_proto::Address;
    use std::borrow::Cow;

    #[test]
    fn parse_address_parses_address_containing_only_name() {
        let address = Address {
            name: Some(std::borrow::Cow::Borrowed(b"Benito camela")),
            adl: None,
            mailbox: None,
            host: None,
        };

        let parsed = parse_address(&[address]);

        assert!(matches!(parsed, Some(value) if value == "Benito camela"));
    }

    #[test]
    fn parse_address_parses_address_containing_name_and_mailbox() {
        let address = Address {
            name: Some(Cow::Borrowed(b"Benito camela")),
            adl: None,
            mailbox: Some(Cow::Borrowed(b"Whatever")),
            host: None,
        };

        let parsed = parse_address(&[address]);

        assert!(matches!(parsed, Some(value) if value == "Benito camela"));
    }

    #[test]
    fn parse_address_parses_address_containing_name_and_host() {
        let address = Address {
            name: Some(Cow::Borrowed(b"Benito camela")),
            adl: None,
            mailbox: None,
            host: Some(Cow::Borrowed(b"pepe.de")),
        };

        let parsed = parse_address(&[address]);

        assert!(matches!(parsed, Some(value) if value == "Benito camela <pepe.de>"));
    }

    #[test]
    fn parse_address_parses_address_containing_only_host() {
        let address = Address {
            name: None,
            adl: None,
            mailbox: None,
            host: Some(Cow::Borrowed(b"pepe.de")),
        };

        let parsed = parse_address(&[address]);

        assert!(matches!(parsed, Some(value) if value == "<pepe.de>"));
    }

    #[test]
    fn parse_address_parses_address_containing_host_and_mailbox() {
        let address = Address {
            name: None,
            adl: None,
            mailbox: Some(Cow::Borrowed(b"mario")),
            host: Some(Cow::Borrowed(b"pepe.de")),
        };

        let parsed = parse_address(&[address]);

        assert!(matches!(parsed, Some(value) if value == "<mario@pepe.de>"));
    }

    #[test]
    fn parse_address_parses_address_containing_name_and_host_and_mailbox() {
        let address = Address {
            name: Some(Cow::Borrowed(b"Mario bros")),
            adl: None,
            mailbox: Some(Cow::Borrowed(b"mario")),
            host: Some(Cow::Borrowed(b"pepe.de")),
        };

        let parsed = parse_address(&[address]);

        assert!(matches!(parsed, Some(value) if value == "Mario bros <mario@pepe.de>"));
    }

    #[test]
    fn parse_address_parses_address_containing_nothing() {
        let address = Address {
            name: None,
            adl: None,
            mailbox: None,
            host: None,
        };

        let parsed = parse_address(&[address]);

        assert_eq!(parsed, None);
    }

    #[test]
    fn parse_address_parses_address_containing_only_mailbox() {
        let address = Address {
            name: None,
            adl: None,
            mailbox: Some(Cow::Borrowed(b"whatever")),
            host: None,
        };

        let parsed = parse_address(&[address]);

        assert_eq!(parsed, None);
    }
}

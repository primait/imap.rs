use super::Flag;
use crate::datetime::IMAP_DATE_FORMAT;
use chrono::NaiveDate;
use derive_more::Display;

/// Static typed query builder for IMAP search queries.
///
/// To learn more about how IMAP queries work check this out:
///
/// <https://datatracker.ietf.org/doc/html/rfc3501#section-6.4.4>
///
/// ```rust
/// use imap_session::{Query, Flag};
/// use chrono::NaiveDate;
///
/// let query = Query::or(
///     Query::flag(Flag::Flagged),
///     Query::or(
///         Query::flag(Flag::Draft),
///         Query::and(
///             Query::flag(Flag::custom("custom flag")),
///             Query::flag(Flag::Seen),
///         ),
///     ),
/// );
///
/// assert_eq!(
///     "OR (FLAGGED) (OR (DRAFT) (KEYWORD custom-flag SEEN))",
///     query.to_string()
/// );
///
/// let query = !Query::flag(Flag::custom("testflag1"));
///
/// assert_eq!("NOT (KEYWORD testflag1)", query.to_string());
///
/// let query = !Query::and(
///     Query::flag(Flag::custom("testflag1")),
///     Query::flag(Flag::Seen),
/// );
///
/// assert_eq!("NOT (KEYWORD testflag1 SEEN)", query.to_string());
///
/// let query = Query::and(
///     !Query::flag(Flag::custom("testflag1")),
///     Query::since(NaiveDate::from_ymd_opt(2024, 1, 12).unwrap()),
/// );
///
/// assert_eq!(
///     "NOT (KEYWORD testflag1) SINCE 12-Jan-2024",
///     query.to_string()
/// );
///
/// let query = Query::and(!Query::flag(Flag::custom("testflag1")), Query::unseen());
///
/// assert_eq!("NOT (KEYWORD testflag1) UNSEEN", query.to_string());
///
/// let query = Query::and(
///     !Query::flag(Flag::custom("testflag1")),
///     Query::before(NaiveDate::from_ymd_opt(2024, 1, 12).unwrap()),
/// );
///
/// assert_eq!(
///     "NOT (KEYWORD testflag1) BEFORE 12-Jan-2024",
///     query.to_string()
/// );
/// ```
#[derive(Debug, Clone, Display)]
#[display("{_0}")]
pub struct Query(Component);

impl Query {
    pub fn flag(value: Flag) -> Query {
        Query(Component::Flag(value))
    }

    pub fn since(value: NaiveDate) -> Query {
        Query(Component::Since(value))
    }

    pub fn before(value: NaiveDate) -> Query {
        Query(Component::Before(value))
    }

    pub fn unseen() -> Query {
        Query(Component::Unseen)
    }

    pub fn and(left: Query, right: Query) -> Query {
        Query(Component::And(Box::new(left.0), Box::new(right.0)))
    }

    pub fn or(left: Query, right: Query) -> Query {
        Query(Component::Or(Box::new(left.0), Box::new(right.0)))
    }
}

impl std::ops::Not for Query {
    type Output = Self;

    fn not(self) -> Self::Output {
        Query(Component::Not(Box::new(self.0)))
    }
}

#[derive(Debug, Clone)]
enum Component {
    And(Box<Component>, Box<Component>),
    Or(Box<Component>, Box<Component>),
    Not(Box<Component>),
    Flag(Flag),
    Unseen,
    Since(NaiveDate),
    Before(NaiveDate),
}

impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Component::And(left, right) => write!(f, "{left} {right}"),
            Component::Or(left, right) => write!(f, "OR ({left}) ({right})"),
            Component::Not(value) => write!(f, "NOT ({value})"),
            Component::Unseen => write!(f, "UNSEEN"),
            Component::Since(value) => write!(f, "SINCE {}", value.format(IMAP_DATE_FORMAT)),
            Component::Before(value) => write!(f, "BEFORE {}", value.format(IMAP_DATE_FORMAT)),
            Component::Flag(flag) => match flag {
                Flag::Seen => write!(f, "SEEN"),
                Flag::Answered => write!(f, "ANSWERED"),
                Flag::Flagged => write!(f, "FLAGGED"),
                Flag::Deleted => write!(f, "DELETED"),
                Flag::Draft => write!(f, "DRAFT"),
                Flag::Recent => write!(f, "RECENT"),
                Flag::MayCreate => write!(f, "KEYWORD *"),
                Flag::Custom(value) => write!(f, "KEYWORD {}", value.trim().replace(' ', "-")),
            },
        }
    }
}

use chrono::{DateTime, NaiveDateTime};

/// Parses a string representing a datetime into an UTC NaiveDateTime.
/// Tries different patterns failing if all the conversions fail.
pub fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    DateTime::parse_from_rfc2822(s)
        .or_else(|_| DateTime::parse_from_rfc3339(s))
        .ok()
        .map(|d| d.naive_utc())
}

pub static IMAP_DATE_FORMAT: &str = "%d-%b-%Y";

#[cfg(test)]
mod test {
    use super::IMAP_DATE_FORMAT;
    use super::parse_datetime;
    use chrono::NaiveDate;

    #[test]
    fn parse_from_rfc2822() {
        let res = parse_datetime("Wed, 6 Jul 2022 16:36:48 +0200 (CEST)");
        assert!(res.is_some());

        let res = parse_datetime("Wed, 6 Jul 2022 16:36:48 +0200");
        assert!(res.is_some());

        let res = parse_datetime("16 Oct 2023 12:44:35 -0400");
        assert!(res.is_some());
    }

    #[test]
    fn parse_from_rfc3339() {
        let res = parse_datetime("1996-12-19T16:39:57-08:00");
        assert!(res.is_some());

        let res = parse_datetime("1937-01-01T12:00:27.87+00:20");
        assert!(res.is_some());
    }

    #[test]
    fn formats_imap_date() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 12).unwrap();
        assert_eq!(date.format(IMAP_DATE_FORMAT).to_string(), "12-Jan-2024");

        let date = NaiveDate::from_ymd_opt(2024, 4, 12).unwrap();
        assert_eq!(date.format(IMAP_DATE_FORMAT).to_string(), "12-Apr-2024");
    }
}

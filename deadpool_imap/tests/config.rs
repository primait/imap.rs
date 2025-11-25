use config::Config;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Configuration {
    #[serde(alias = "imap__host")]
    pub host: String,
    #[serde(alias = "imap__port")]
    pub imap_port: u16,
    #[serde(alias = "greenmail__api_port")]
    pub greenmail_port: u16,
    #[serde(alias = "smtp__port")]
    pub smtp_port: u16,
}

impl Configuration {
    pub fn load() -> Self {
        Config::builder()
            .add_source(config::Environment::default())
            .build()
            .expect("Error building Configuration")
            .try_deserialize()
            .expect("Error trying to deserialize")
    }
}

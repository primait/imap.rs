use fake::faker::internet::en::{Password, Username};
use std::collections::HashMap;
use url::Url;

pub struct GreenmailClient {
    reqwest: reqwest::Client,
    host: String,
    api_port: u16,
}

#[derive(fake::Dummy)]
pub struct User {
    #[dummy(faker = "Username()")]
    pub username: String,
    #[dummy(faker = "Password(5..10)")]
    pub password: String,
}

impl User {
    pub fn email(&self) -> String {
        format!("{}@localhost", self.username)
    }
}

impl GreenmailClient {
    pub fn new(host: String, api_port: u16) -> Self {
        Self {
            host,
            api_port,
            reqwest: reqwest::Client::new(),
        }
    }

    fn base_url(&self) -> Url {
        Url::parse(&format!("http://{}:{}", self.host, self.api_port)).unwrap()
    }

    pub async fn create_user(&self, user: &User) {
        let body = vec![
            ("login", user.username.clone()),
            ("password", user.password.clone()),
            ("email", user.email()),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let url = self.base_url().join("/api/user").unwrap();

        let _ = self
            .reqwest
            .post(url)
            .json(&body)
            .send()
            .await
            .expect("Error creating greenmail user");
    }

    pub async fn delete_user(&self, user: &User) {
        let url = self
            .base_url()
            .join(&format!("api/user/{}", user.username))
            .unwrap();

        let _ = self
            .reqwest
            .delete(url)
            .send()
            .await
            .expect("Error creating greenmail user");
    }
}

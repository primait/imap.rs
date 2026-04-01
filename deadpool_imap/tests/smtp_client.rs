use fake::Fake;
use fake::faker::lorem::en::Sentence;
use lettre::Message;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials as LettreCredentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

#[derive(Clone, Debug)]
pub struct SmtpClient {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    user: String,
}

type ServerAddress = (String, u16);

#[derive(Debug)]
pub struct Credentials {
    pub user: String,
    pub password: String,
}

#[derive(Clone)]
pub struct SendEmailCommand {
    pub to: String,
    pub body: String,
    pub subject: String,
}

pub struct SendEmailCommandFaker(pub String);

impl fake::Dummy<SendEmailCommandFaker> for SendEmailCommand {
    fn dummy_with_rng<R: fake::rand::Rng + ?Sized>(
        config: &SendEmailCommandFaker,
        _rng: &mut R,
    ) -> Self {
        Self {
            to: config.0.clone(),
            body: Sentence(10..20).fake(),
            subject: Sentence(10..20).fake(),
        }
    }
}

impl SmtpClient {
    pub async fn new((domain, port): ServerAddress, credentials: Credentials) -> Self {
        let creds = LettreCredentials::new(credentials.user.clone(), credentials.password);

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(domain)
                .credentials(creds)
                .port(port)
                .timeout(Some(std::time::Duration::from_secs(5)))
                .build();

        mailer
            .test_connection()
            .await
            .expect("Couldn't connect to SMTP server");

        Self {
            transport: mailer,
            user: credentials.user,
        }
    }

    pub async fn send_email(&self, data: SendEmailCommand) {
        let from = format!("{0}@localhost", self.user.clone());

        let email = Message::builder()
            .from(from.parse().unwrap())
            .to(data.to.parse().unwrap())
            .subject(data.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(data.body)
            .unwrap();

        // Send the email
        let _ = self
            .transport
            .send(email)
            .await
            .expect("Error sending email");
    }
}

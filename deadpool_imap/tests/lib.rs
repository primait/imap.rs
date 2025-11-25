mod config;
mod greenmail;
mod smtp_client;

use self::{
    config::Configuration as TestConfig,
    greenmail::{GreenmailClient, User},
};
use crate::smtp_client::{SendEmailCommand, SendEmailCommandFaker};
use async_native_tls::TlsConnector;
use deadpool::managed;
use deadpool_imap::ImapConnectionManager;
use fake::{Fake, Faker};
use imap_session::{ConnectionConfig, Credentials, Flag, Message, Query, Uid};
use smtp_client::SmtpClient;
use std::collections::HashSet;
use test_context::{AsyncTestContext, test_context};

type ImapPool = managed::Pool<ImapConnectionManager>;

struct Context {
    smtp_client: SmtpClient,
    imap_pool: ImapPool,
    greenmail_client: GreenmailClient,
    user: User,
}

impl AsyncTestContext for Context {
    async fn setup() -> Self {
        let config = TestConfig::load();

        let greenmail_client = GreenmailClient::new(config.host.clone(), config.greenmail_port);

        let user: User = Faker.fake();
        greenmail_client.create_user(&user).await;

        let smtp_client = SmtpClient::new(
            (config.host.clone(), config.smtp_port),
            smtp_client::Credentials {
                user: user.username.clone(),
                password: user.password.clone(),
            },
        )
        .await;

        let pool_size = 1;
        let manager = ImapConnectionManager::new(
            ConnectionConfig {
                domain: config.host.clone(),
                port: config.imap_port,
                credentials: Credentials {
                    user: user.username.clone(),
                    password: user.password.clone(),
                },
            },
            pool_size,
            || TlsConnector::new().danger_accept_invalid_certs(true),
        );

        let imap_pool = ImapPool::builder(manager)
            .max_size(pool_size)
            .build()
            .expect("Can't build the connection pool");

        Self {
            smtp_client,
            imap_pool,
            greenmail_client,
            user,
        }
    }

    async fn teardown(self) {
        self.greenmail_client.delete_user(&self.user).await;
    }
}

#[test_context(Context)]
#[tokio::test(flavor = "multi_thread")]
async fn searches_unseen_emails(ctx: &mut Context) {
    ctx.smtp_client
        .send_email(SendEmailCommandFaker(ctx.user.email()).fake())
        .await;

    let uids = ctx
        .imap_pool
        .get()
        .await
        .unwrap()
        .search("INBOX", !Query::flag(Flag::Seen))
        .await
        .expect("Error searching emails");

    assert_eq!(uids.len(), 1);
}

#[test_context(Context)]
#[tokio::test(flavor = "multi_thread")]
async fn searches_seen_emails(ctx: &mut Context) {
    ctx.smtp_client
        .send_email(SendEmailCommandFaker(ctx.user.email()).fake())
        .await;

    let uids = ctx
        .imap_pool
        .get()
        .await
        .unwrap()
        .search("INBOX", Query::flag(Flag::Seen))
        .await
        .expect("Error searching emails");

    assert_eq!(uids.len(), 0);
}

#[test_context(Context)]
#[tokio::test(flavor = "multi_thread")]
async fn can_set_standard_flags(ctx: &mut Context) {
    ctx.smtp_client
        .send_email(SendEmailCommandFaker(ctx.user.email()).fake())
        .await;

    let mut imap_client = ctx.imap_pool.get().await.unwrap();

    let uids = imap_client
        .search("INBOX", !Query::flag(Flag::Seen))
        .await
        .expect("Error searching emails");

    assert_eq!(uids.len(), 1);

    imap_client
        .set_flags("INBOX", uids.to_vec(), vec![Flag::Draft])
        .await
        .expect("Error setting flags");

    let uids = imap_client
        .search("INBOX", Query::flag(Flag::Draft))
        .await
        .expect("Error searching emails");

    assert_eq!(uids.len(), 1);
}

#[test_context(Context)]
#[tokio::test(flavor = "multi_thread")]
async fn can_set_custom_flags(ctx: &mut Context) {
    ctx.smtp_client
        .send_email(SendEmailCommandFaker(ctx.user.email()).fake())
        .await;

    ctx.smtp_client
        .send_email(SendEmailCommandFaker(ctx.user.email()).fake())
        .await;

    let mut imap_client = ctx.imap_pool.get().await.unwrap();

    let uids = imap_client
        .search("INBOX", !Query::flag(Flag::Seen))
        .await
        .expect("Error searching emails");

    let flag = Flag::custom(Faker.fake::<String>());

    imap_client
        .set_flags("INBOX", uids.to_vec(), vec![flag.clone()])
        .await
        .expect("Error setting flags");

    let uids = imap_client
        .search("INBOX", Query::flag(flag))
        .await
        .expect("Error searching emails");

    assert_eq!(uids.len(), 2);
}

#[test_context(Context)]
#[tokio::test(flavor = "multi_thread")]
async fn can_search_messages_containing_either_of_two_flags(ctx: &mut Context) {
    ctx.smtp_client
        .send_email(SendEmailCommandFaker(ctx.user.email()).fake())
        .await;

    ctx.smtp_client
        .send_email(SendEmailCommandFaker(ctx.user.email()).fake())
        .await;

    ctx.smtp_client
        .send_email(SendEmailCommandFaker(ctx.user.email()).fake())
        .await;

    let mut imap_client = ctx.imap_pool.get().await.unwrap();

    let uids = imap_client
        .search("INBOX", !Query::flag(Flag::Seen))
        .await
        .expect("Error searching emails");

    let flag1 = Flag::custom(Faker.fake::<String>());
    let flag2 = Flag::custom(Faker.fake::<String>());

    imap_client
        .set_flags("INBOX", vec![uids[0]], vec![flag1.clone()])
        .await
        .expect("Error setting flags");

    imap_client
        .set_flags("INBOX", vec![uids[1]], vec![flag2.clone()])
        .await
        .expect("Error setting flags");

    let flagged_uids = imap_client
        .search(
            "INBOX",
            Query::or(Query::flag(flag1.clone()), Query::flag(flag2.clone())),
        )
        .await
        .expect("Error searching emails");

    assert_eq!(flagged_uids.len(), 2);

    assert_eq!(
        HashSet::<Uid>::from_iter(flagged_uids.into_iter()),
        HashSet::from_iter(vec![uids[0], uids[1]].into_iter())
    );
}

#[test_context(Context)]
#[tokio::test(flavor = "multi_thread")]
async fn can_fetch_messages(ctx: &mut Context) {
    let command = SendEmailCommand {
        body: "Hello World".to_owned(),
        ..SendEmailCommandFaker(ctx.user.email()).fake()
    };

    ctx.smtp_client.send_email(command.clone()).await;

    let mut imap_client = ctx.imap_pool.get().await.unwrap();

    let uids = imap_client
        .search("INBOX", !Query::flag(Flag::Seen))
        .await
        .expect("Error searching emails");

    let first_uid = uids[0];

    let fetched_message = imap_client
        .fetch_one("INBOX", first_uid)
        .await
        .expect("Error fetching messages");

    let Message {
        flags: _flags,
        uid,
        body,
        subject,
        from,
        to,
        cc,
        send_date,
        received_date,
    } = fetched_message;

    assert_eq!(first_uid, uid);
    assert_eq!(subject.clone().unwrap(), command.subject.into());
    assert_eq!(
        from.clone().unwrap(),
        format!("<{}>", ctx.user.email()).into()
    );
    assert!(send_date.is_some());
    assert!(received_date.is_some());
    assert_eq!(&*to.unwrap(), format!("<{}>", command.to));
    assert!(cc.is_none());

    assert!(
        String::from_utf8(body.to_vec())
            .unwrap()
            .contains(&command.body)
    );
}

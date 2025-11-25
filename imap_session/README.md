# imap_session 🦀: Core Async IMAP Session Management for Rust

[![Crate Version: imap_session](https://img.shields.io/crates/v/imap_session.svg)](https://crates.io/crates/imap_session) 
[![License](https://img.shields.io/badge/License-MIT%2FApache%202.0-blue.svg)](../LICENSE)

`imap_session` is the **core library** of the `imap.rs` workspace. It provides a robust, idiomatic Rust wrapper built on top of `async_imap` for managing and performing operations on a single, secure IMAP session.

This crate handles the complexities of connection configuration, TLS handshake, and authentication, making it suitable for short-lived or singular IMAP tasks.

---

## 📦 Installation

To use `imap_session` in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
imap_session = "0.1" 
tokio = { version = "1", features = ["full"] }
async-native-tls = { version = "0.5.0", default-features = false, features = [
  "tokio",
  "runtime-tokio",
  "futures-util",
] }
````

-----

## 🚀 Usage Example (Single Session)

Use `imap_session` when you only need to open, use, and close a single connection, typically for short-lived tasks like fetching a single piece of data or running a simple command.

```rust
use async_native_tls::TlsConnector;
use imap_session::{ConnectionConfig, Credentials, Query, Flag};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {   
    let config = ConnectionConfig {
        domain: std::env::var("IMAP__HOST").unwrap(),
        port: std::env::var("IMAP__PORT").unwrap().parse().unwrap(),
        credentials: Credentials {
            user: "test".to_string(),
            password: "test".to_string(),
        },
    };

    // Do not use `danger_accept_invalid_certs(true)` in production
    let tls_connector = TlsConnector::new().danger_accept_invalid_certs(true);

    // The connect function handles the TCP connection, TLS upgrade, and login.
    let mut session = imap_session::connect(&config, tls_connector).await?;

    println!("Successfully connected and logged in.");

    // Perform an IMAP search operation
    // Search for messages that are NOT yet marked as Seen
    let uids = session.search("INBOX", !Query::flag(Flag::Seen)).await?;
    
    println!("INBOX contains {} unseen messages.", uids.len());

    // Consumes the session and sends the LOGOUT command
    session.logout().await?;

    Ok(())
}
```

-----

## 📄 License

This project is licensed under:

  * [MIT license](../LICENSE)

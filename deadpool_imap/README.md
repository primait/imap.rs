# deadpool_imap 🌊: High-Performance IMAP Connection Pooling for Rust

[![Crate Version: deadpool_imap](https://img.shields.io/crates/v/deadpool_imap.svg)](https://crates.io/crates/deadpool_imap) 
[![License](https://img.shields.io/badge/License-MIT%2FApache%202.0-blue.svg)](../LICENSE)

`deadpool_imap` provides a highly efficient and reliable **IMAP connection pool** implementation built on the `deadpool` framework and the core **`imap_session`** crate.

This crate is ideal for server applications, APIs, or any service where IMAP sessions need to be frequently opened and reused across multiple concurrent requests, drastically reducing latency by avoiding repeated TLS handshakes and authentication.

---

## 📦 Installation

To use `deadpool_imap` in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
deadpool_imap = "0.1" 
imap_session = "0.1" 
tokio = { version = "1", features = ["full"] }
deadpool = { version = "0.12", features = ["managed"] } 
async-native-tls = { version = "0.5.0", default-features = false, features = [
  "tokio",
  "runtime-tokio",
  "futures-util",
] }
````

-----

## 🚀 Usage Example (Pooled Sessions)

Use the pool for server applications or APIs where IMAP sessions need to be reused across multiple requests to reduce latency and resource consumption.

```rust
use async_native_tls::TlsConnector;
use deadpool::managed;
use deadpool_imap::ImapConnectionManager;
use imap_session::{ConnectionConfig, Credentials, Query, Flag};

type ImapPool = managed::Pool<ImapConnectionManager>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const POOL_SIZE: usize = 5;

    // The manager requires the connection config, pool size (for connection limits), 
    // and a factory function for the TLS connector.
    let manager = ImapConnectionManager::new(
        ConnectionConfig {
            domain: std::env::var("IMAP__HOST").unwrap(),
            port: std::env::var("IMAP__PORT").unwrap().parse().unwrap(),
            credentials: Credentials {
                user: "test".to_string(),
                password: "test".to_string(),
            },
        },
        POOL_SIZE,
        // Do not use `danger_accept_invalid_certs(true)` in production
        || TlsConnector::new().danger_accept_invalid_certs(true),
    );

    // Build the pool using the manager
    let imap_pool = ImapPool::builder(manager)
        .max_size(POOL_SIZE)
        .build()?;

    println!("IMAP Connection Pool initialized with max size {}.", POOL_SIZE);

    // Acquire a session from the pool. This session is automatically recycled on Drop.
    let mut session = imap_pool.get().await.unwrap();

    // Perform an IMAP search operation
    // Search for messages that are NOT yet marked as Seen
    let uids = session.search("INBOX", !Query::flag(Flag::Seen)).await?;

    println!("INBOX contains {} unseen messages.", uids.len());

    // When `session` goes out of scope, the underlying connection is automatically 
    // returned to the pool, ready for the next request.
    
    Ok(())
}
```

-----

## 📄 License

This project is licensed under:

  * [MIT license](../LICENSE)

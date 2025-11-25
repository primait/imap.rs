# imap.rs 🦀: Asynchronous IMAP Client and Connection Pool

[![Crate Version: deadpool_imap](https://img.shields.io/crates/v/deadpool_imap.svg)](https://crates.io/crates/deadpool_imap)
[![Crate Version: imap_session](https://img.shields.io/crates/v/imap_session.svg)](https://crates.io/crates/imap_session)
[![License](https://img.shields.io/badge/License-MIT%2FApache%202.0-blue.svg)](LICENSE)

`imap.rs` is a repository providing two high-level crates for interacting with IMAP servers asynchronously. It is built on top of the `async_imap` library.

-----

## 🏗️ Crates Overview

This repository contains the following two independent crates. Click the links below for detailed installation instructions and usage examples for each.

### 1\. `imap_session` (Core IMAP Client)

The fundamental building block for interacting with an IMAP server.

  * **Purpose:** Handles all aspects of establishing a single, secure IMAP session: TCP connection, TLS negotiation, and user authentication (login).
  * **Best For:** Short-lived tasks, simple scripts, or applications where connection reuse is not critical.
  * **Documentation:** [View `imap_session` README](./imap_session/README.md)

### 2\. `deadpool_imap` (Connection Pool)

A production-grade solution for managing concurrent connections.

  * **Purpose:** Provides a high-performance, asynchronous connection pool using the `deadpool` crate. It intelligently manages connections, performing recycling and health checks to ensure reliability.
  * **Best For:** Server applications, web APIs, or microservices that require high concurrency and frequent, low-latency access to the IMAP server.
  * **Documentation:** [View `deadpool_imap` README](./deadpool_imap/README.md)

-----

## 📄 License

This repository is licensed under the MIT license.

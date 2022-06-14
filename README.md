# TigronSms-Rust

## Description
This module allows you to send text messages using Tigron's SMS-API.

## Installation
1. Download the file tigron_sms.rs and include the module in your main.rs
2. Append the dependencies in Cargo.toml to your project.

## Requirements
- A Tigron account and the purchased SMS product.
- The Rust Language (2018 edition)
- Tokio (v0.2)
- An asynchronous project

## Example

```rust
mod tigron_sms;

#[tokio::main]
async fn main() {
    let tigron_sms = tigron_sms::TigronSms {
        credentials: ("YOUR_TIGRON_USERNAME".to_string(), "YOUR_TIGRON_PASSWORD".to_string()),
    };

    let to = "+32.xxxxxxxxx".to_string();
    let from = "+32.xxxxxxxxx".to_string();
    let message = "Hello world!".to_string();

    if let Err(e) = tigron_sms.send(to, from, message).await {
        eprintln!("{}", e);
    }
}

```

## Todo
- Clean up code.
- Documentation.
- Input validation (telephone number must be in valid format, message length must be under 160,...)

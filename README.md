# identity_dbus_broker

`identity_dbus_broker` is a Rust crate that provides traits for implementing D-Bus services used in Azure Entra ID authentication. It is part of the larger Himmelblau project, aimed at building an identity broker compatible with Microsoft's proprietary Linux Intune Portal. This crate offers two main traits, `SessionBroker` and `DeviceBroker`, which represent the session D-Bus service `com.microsoft.identity.broker1` and system service `com.microsoft.identity.DeviceBroker1`, respectively. These services allow for the handling of authentication requests, similar to what Microsoft's proprietary Intune binaries provide.

## Features

- **SessionBroker**: Handles D-Bus requests related to session authentication.
- **DeviceBroker**: Manages device-related authentication.
- **HimmelblauBroker**: Includes a session service implementation that forwards `SessionBroker` requests to the HimmelblauBroker system D-Bus service, located at `org.samba.himmelblau`.

The traits provided by this crate simplify the implementation of these D-Bus services.

## Example Usage

### Himmelblau Session Broker Example

The following is an example of how to use the `identity_dbus_broker` crate to start a session broker service that forwards requests to the HimmelblauBroker system service.

```rust
use identity_dbus_broker::himmelblau_session_broker_serve;

#[tokio::main]
async fn main() -> Result<(), dbus::MethodErr> {
    himmelblau_session_broker_serve().await
}
```

### Himmelblau System Broker Example

This example demonstrates how to implement the `HimmelblauBroker` trait to handle system-level authentication requests.

```rust
use identity_dbus_broker::{HimmelblauBroker, himmelblau_broker_serve};
use libc::uid_t;

struct Broker;

impl HimmelblauBroker for Broker {
    fn acquire_token_silently(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr> {
        eprintln!("The peer uid is {}", uid);
        Ok("{
            \"access_token\": \"eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsIng1dCI6Imk2bEdrM0ZaenhSY1ViMkMzbkVRN3N5SEpsWSIsImtpZCI6Imk2bEdrM0ZaenhSY1ViMkMzbkVRN3N5SEpsWSJ9.eyJhdWQiOiJlZjFkYTlkNC1mZjc3LTRjM2UtYTAwNS04NDBjM2Y4MzA3NDUiLCJpc3MiOiJodHRwczovL3N0cy53aW5kb3dzLm5ldC9mYTE1ZDY5Mi1lOWM3LTQ0NjAtYTc0My0yOWYyOTUyMjIyOS8iLCJpYXQiOjE1MzcyMzMxMDYsIm5iZiI6MTUzNzIzMzEwNiwiZXhwIjoxNTM3MjM3MDA2LCJhY3IiOiIxIiwiYWlvIjoiQVhRQWkvOElBQUFBRm0rRS9RVEcrZ0ZuVnhMaldkdzhLKzYxQUdyU091TU1GNmViYU1qN1hPM0libUQzZkdtck95RCtOdlp5R24yVmFUL2tES1h3NE1JaHJnR1ZxNkJuOHdMWG9UMUxrSVorRnpRVmtKUFBMUU9WNEtjWHFTbENWUERTL0RpQ0RnRTIyMlRJbU12V05hRU1hVU9Uc0lHdlRRPT0iLCJhbXIiOlsid2lhIl0sImFwcGlkIjoiNzVkYmU3N2YtMTBhMy00ZTU5LTg1ZmQtOGMxMjc1NDRmMTdjIiwiYXBwaWRhY3IiOiIwIiwiZW1haWwiOiJBYmVMaUBtaWNyb3NvZnQuY29tIiwiZmFtaWx5X25hbWUiOiJMaW5jb2xuIiwiZ2l2ZW5fbmFtZSI6IkFiZSAoTVNGVCkiLCJpZHAiOiJodHRwczovL3N0cy53aW5kb3dzLm5ldC83MmY5ODhiZi04NmYxLTQxYWYtOTFhYi0yZDdjZDAxMjIyNDcvIiwiaXBhZGRyIjoiMjIyLjIyMi4yMjIuMjIiLCJuYW1lIjoiYWJlbGkiLCJvaWQiOiIwMjIyM2I2Yi1hYTFkLTQyZDQtOWVjMC0xYjJiYjkxOTQ0MzgiLCJyaCI6IkkiLCJzY3AiOiJ1c2VyX2ltcGVyc29uYXRpb24iLCJzdWIiOiJsM19yb0lTUVUyMjJiVUxTOXlpMmswWHBxcE9pTXo1SDNaQUNvMUdlWEEiLCJ0aWQiOiJmYTE1ZDY5Mi1lOWM3LTQ0NjAtYTc0My0yOWYyOTU2ZmQ0MjkiLCJ1bmlxdWVfbmFtZSI6ImFiZWxpQG1pY3Jvc29mdC5jb20iLCJ1dGkiOiJGVnNHeFlYSTMwLVR1aWt1dVVvRkFBIiwidmVyIjoiMS4wIn0.D3H6pMUtQnoJAGq6AHd\",
            \"token_type\": \"Bearer\",
            \"expires_in\": 5368
        }"
        .to_string())
    }

    fn get_accounts(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr> {
        eprintln!("The peer uid is {}", uid);
        Ok("{
            \"accounts\": [
                {
                    \"username\": \"tux@test.onmicrosoft.com\",
                    \"realm\": \"29a02212-775d-43b2-a073-a265002c7d11\"
                }
            ]
        }"
        .to_string())
    }

    // Other methods omitted for brevity
}

#[tokio::main]
async fn main() -> Result<(), dbus::MethodErr> {
    himmelblau_broker_serve::<Broker>(Broker {}).await
}
```

## Getting Started

To use this crate, add the following to your `Cargo.toml`:

```toml
[dependencies]
identity_dbus_broker = "0.1.0"
```

Then, implement the required traits (`SessionBroker` and/or `DeviceBroker`) for your project. You can start the respective D-Bus service using the provided functions:

- `session_broker_serve()`
- `device_broker_serve()`

## Licensing

`identity_dbus_broker` is licensed under the LGPL-3.0 license, making it suitable for use in both open source and proprietary projects.

## Contributions

Contributions are welcome! If you'd like to collaborate or propose improvements, feel free to open an issue or a pull request.

For more information, check out the [Himmelblau project](https://github.com/himmelblau-idm/himmelblau).
```

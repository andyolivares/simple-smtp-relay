# Simple SMTP Relay
Application written in Rust that listens for clients, reads e-mails and then delivers them using [Azure Communication Services](https://azure.microsoft.com/en-us/products/communication-services).

The server is inspired by the work done by [Piotr Sarna](https://blog.turso.tech/write-your-own-email-server-in-rust-36f4ff5b1956).

# Usage
```bash
$ ./simple_smtp_relay [<bind_address>:<port>] [<domain>]
```

By default, it binds to `127.0.0.1:25` with domain `smtp.domain.com`

Example:

```bash
$ ./simple_stmp_relay 127.0.0.1:2525 mydomain.com
```

# Logging

The server uses standard Rust log library and uses `RUST_LOG` environment variable to control logging level (trace, debug, info, warn, error).

```bash
$ RUST_LOG=debug ./simple_smtp_relay
```

# Azure Communication Services

For e-mail delivery through ACS, two environment variables need to be set: `ACS_ENDPOINT` and `ACS_ACCESS_KEY`

```bash
$ RUST_LOG=debug ACS_ENDPOINT=https://<my_acs_account>.unitedstates.communication.azure.com ACS_ACCESS_KEY=<my_access_key> ./simple_smtp_relay
```

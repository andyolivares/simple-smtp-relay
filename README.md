# Simple SMTP Relay
Application written in Rust that listens for SMTP clients, reads e-mails and then delivers them using [Azure Communication Services](https://azure.microsoft.com/en-us/products/communication-services).

The server is inspired by the work done by [Piotr Sarna](https://blog.turso.tech/write-your-own-email-server-in-rust-36f4ff5b1956).

```
Usage: simple-smtp-relay.exe [OPTIONS] --endpoint <ACS_ENDPOINT> --access-key <ACS_ACCESS_KEY>

Options:
  -a, --address <IP:PORT>            Address to bind to (defaults to 127.0.0.1:25)
  -d, --domain <DOMAIN>              Domain name (defaults to smtp.domain.com)
  -e, --endpoint <ACS_ENDPOINT>      ACS Endpoint (eg. https://<my_acs_account>.unitedstates.communication.azure.com)
  -k, --access-key <ACS_ACCESS_KEY>  ACS Access Key
  -l, --log-level <LOG_LEVEL>        Log level (eg. trace, debug, info, warn, error)
  -h, --help                         Print help
  -V, --version                      Print version
```
# Example `systemd` Daemon Configuration File

```
[Unit]
Description=Simple SMTP Relay Server

[Service]
Environment=ACS_ENDPOINT=https://<my_acs_account>.unitedstates.communication.azure.com
Environment=ACS_ACCESS_KEY=<my_access_key>
ExecStart=/usr/local/bin/simple-smtp-relay -e $ACS_ENDPOINT -k $ACS_ACCESS_KEY

[Install]
WantedBy=multi-user.target
```
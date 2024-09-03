# Simple SMTP Relay

Application written in Rust that listens for SMTP clients, reads e-mails and then delivers them using [Azure Communication Services](https://azure.microsoft.com/en-us/products/communication-services).

The server has been inspired by the work done by [Piotr Sarna](https://blog.turso.tech/write-your-own-email-server-in-rust-36f4ff5b1956).

## Installation

You need to install a Rust toolchain with `cargo`. Please refer to [Install Rust](https://www.rust-lang.org/tools/install) for up-to-date instructions for your platform.

Once you have `cargo` up and running, execute the following command:

```bash
$ sudo cargo install --git https://github.com/andyolivares/simple-smtp-relay.git --root /usr/local
```

This will download, build and install `simple-smtp-relay` to `/usr/local/bin`.

## Usage

```
Usage: simple-smtp-relay [OPTIONS] --endpoint <ACS_ENDPOINT> --access-key <ACS_ACCESS_KEY>

Options:
  -a, --address <IP:PORT>            Address to bind to (defaults to 127.0.0.1:25)
  -d, --domain <DOMAIN>              Domain name (defaults to smtp.domain.com)
  -e, --endpoint <ACS_ENDPOINT>      ACS Endpoint (eg. https://<my_acs_account>.unitedstates.communication.azure.com)
  -k, --access-key <ACS_ACCESS_KEY>  ACS Access Key
  -l, --log-level <LOG_LEVEL>        Log level (eg. trace, debug, info, warn, error)
  -h, --help                         Print help
  -V, --version                      Print version
```

*WARNING:* Setting the log level to `trace` could reveal sensitive information. Never set to `trace` in production.

## Configure as `systemd` Daemon

If you want to use `systemd` to manage the application as a system daemon, you need to create a `systemd` service file.

```bash
$ sudo editor /etc/systemd/system/ssrelay.service
```

Then type in the following:

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

Don't forget to replace the right values for `ACS_ENDPOINT` and `ACS_ACCESS_KEY`. Once you have the service file in place, reload `systemd` and start the service. You can add any other option in the `ExecStart` command line (eg. another address/port to bind to, domain name, log level, etc.).

Everytime you modify the service file, you need to reload `systemd`:

```bash
$ sudo systemctl daemon-reload
```

Then enable and start the service:

```bash
$ sudo systemctl enable ssrelay
$ sudo systemctl start ssrelay
$ sudo systemctl status ssrelay
```

At this point, you should see the service up and running (STARTED). To see live service logs, you can run:

```bash
$ sudo journalctl -u ssrelay -f
```
# schedules-direct-rs

*Work in progress*

```
RUST_LOG=info cargo run --example sample
```

Confirmed to work on Linux and MacOS. Should work on everything else...

TODO - SQLite DB ingest

# prerequisites

```
cargo install diesel_cli --no-default-features --features sqlite --force
```

## Schedules Direct Grabber

### Enable Log output

```
RUST_LOG=debug
```

### Login Credentials

```
export SD_USER=your_sd_username
export SD_PWD=your_sd_password
```
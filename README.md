# schedules-direct-rs

Work in progress

TODO - SQLite DB ingest

Only testing on Linux.  Should work on everything else...

## Schedules Direct Grabber

Using API 20191022

### Features
Safe Rust
All API calls implement exponential/back-off/retry

### Enable Log output

RUST_LOG=debug

### Login Credentials

    export SD_USER=your_sd_username

    export SD_PWD=your_sd_password

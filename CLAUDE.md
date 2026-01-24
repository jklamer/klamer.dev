# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Style Guidance

Aim to be correct, concise, and consistent with the code you generate. If you are unsure if you should keep generating more, ask.

## Build & Run Commands

```bash
# Build
cargo build --release

# Run locally (port 3000, no TLS)
cargo run

# Run with TLS (deployed mode)
DEPLOYED_ENV=1 cargo run -- -d example.com -b s3-bucket-name --prod

# Run tests
cargo test

# Docker build (ARM64)
docker buildx build . -t <repo>:tag --platform linux/arm64 --push
```

## Architecture

Personal website built with Rust/Axum. Three workspace crates:

- **klamer_dev/** - Main web server using Axum with embedded static content
- **blog_files_macro/** - Proc macro that embeds blog HTML files at compile time as `BLOG_POST_CONTENT`
- **rustls_acme_cache/** - S3-backed Let's Encrypt certificate caching

### Key Design Decisions

- **All static content is embedded** - Images, CSS, HTML pages compiled into binary via `include_bytes!`/`include_str!`
- **TLS handled automatically** - rustls-acme provisions Let's Encrypt certs, S3 stores them between deployments
- **Type-safe HTML generation** - `html.rs` uses builder pattern with derive_builder for HTML elements
- **Two server modes**: Local dev (port 3000, no TLS) vs deployed (ports 80/443 with ACME)

### Routes

`/` home, `/blog` index, `/blog/{name}` posts, `/good_reads`, `/annie` easter egg, static assets, ACME challenges

### Environment

- `DEPLOYED_ENV` - Set to enable TLS mode
- CLI args for TLS: `-d domains`, `-b bucket`, `-e email`, `--prod` for production LE

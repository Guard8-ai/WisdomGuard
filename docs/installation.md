# Installation

## Pre-built Binaries (recommended)

Download the latest release for your platform from the [releases page](https://github.com/Guard8-ai/WisdomGuard/releases):

| Platform | File |
|----------|------|
| Linux x86_64 | `wisdomguard-linux-x86_64` |
| macOS ARM64 | `wisdomguard-macos-aarch64` |
| Windows x86_64 | `wisdomguard-windows-x86_64.exe` |

```bash
# Linux / macOS
chmod +x wisdomguard-linux-x86_64
mv wisdomguard-linux-x86_64 ~/.local/bin/wisdomguard

# Verify
wisdomguard --version
```

## From Source

Requires Rust 1.80+.

```bash
git clone https://github.com/Guard8-ai/WisdomGuard
cd WisdomGuard
cargo install --path .
```

## Prerequisites

WisdomGuard calls the Google VertexAI Gemini API. You need:

1. A Google Cloud project with the **Vertex AI API** enabled.
2. Application Default Credentials (ADC) configured — see [Authentication](authentication.md).

## Verify Installation

```bash
wisdomguard --version
wisdomguard --help
```

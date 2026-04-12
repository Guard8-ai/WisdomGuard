# WisdomGuard

Enhance agentic AI guides with Google VertexAI Gemini insights — adds workflows, gotchas, prioritization, and error solutions.

## Install

```bash
cargo install --path .
```

## Prerequisites

- Google Cloud project with Vertex AI API enabled
- Application Default Credentials configured:
  ```bash
  gcloud auth application-default login
  ```

## Usage

```bash
# Standalone mode: generate enhancement-only guide
wisdomguard cargo-ir.json --project my-gcp-project

# Merge mode: enhance an existing base guide (recommended)
wisdomguard cargo-ir.json --base-guide cargo-guide.md --project my-gcp-project

# Dry run: show prompts without calling API
wisdomguard cargo-ir.json --dry-run

# Custom model and location
wisdomguard api-ir.json --model gemini-2.5-pro --location europe-west1

# JSON output
wisdomguard cargo-ir.json --output-format json -o enhancements.json
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GOOGLE_CLOUD_PROJECT` | GCP project ID | (required) |
| `VERTEX_AI_LOCATION` | Vertex AI region | `us-central1` |
| `VERTEX_AI_MODEL` | Gemini model | `gemini-2.5-flash` |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to service account key | ADC |

## What It Adds

| Section | Description |
|---------|-------------|
| **Common Workflows** | Multi-step task recipes (3-7 items) |
| **Common Mistakes** | Wrong vs Right patterns with explanations |
| **Key Commands** | Most important commands/endpoints (top 20%) |
| **Error Messages** | Common errors with solutions |

## Full Pipeline

```bash
# CLI tool guide
cliguard cargo -o cargo-guide.md
cliguard cargo --format json -o cargo-ir.json
wisdomguard cargo-ir.json --base-guide cargo-guide.md -o AGENTIC_AI_CARGO_GUIDE.md

# API guide
apiguard spec.json -o api-guide.md
apiguard spec.json --output-format json -o api-ir.json
wisdomguard api-ir.json --base-guide api-guide.md -o AGENTIC_AI_API_GUIDE.md

# Documentation guide
docguard src/ -o doc-guide.md
docguard src/ --output-format json -o doc-ir.json
wisdomguard doc-ir.json --base-guide doc-guide.md -o AGENTIC_AI_DOC_GUIDE.md
```

## License

MIT — Guard8.ai

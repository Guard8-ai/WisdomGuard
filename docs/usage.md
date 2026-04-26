# Usage & CLI Reference

## Synopsis

```
wisdomguard <ir_file> [OPTIONS]
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<ir_file>` | Path to the JSON IR file produced by `apiguard --output-format json`, `cliguard --format json`, or `docguard --output-format json` |

## Options

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--base-guide <FILE>` | — | — | Existing `.md` guide to merge enhancements into (merge mode) |
| `--model <MODEL>` | — | `gemini-2.5-flash` | Gemini model to use |
| `--project <PROJECT>` | — | `$GOOGLE_CLOUD_PROJECT` | GCP project ID |
| `--location <REGION>` | — | `$VERTEX_AI_LOCATION` / `us-central1` | VertexAI region |
| `--output <FILE>` | `-o` | stdout | Write output to file |
| `--output-format <FMT>` | — | `md` | Output format: `md` (enhanced guide) or `json` (raw enhancements) |
| `--dry-run` | — | false | Print the LLM prompt without calling the API |
| `--help` | `-h` | — | Print help |
| `--version` | `-V` | — | Print version |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `GOOGLE_CLOUD_PROJECT` | GCP project ID | required if `--project` not set |
| `VERTEX_AI_LOCATION` | VertexAI region | `us-central1` |
| `VERTEX_AI_MODEL` | Gemini model override | `gemini-2.5-flash` |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to service account key file | uses Application Default Credentials |

## Examples

```bash
# Merge mode: enhance existing guide (recommended)
wisdomguard cargo-ir.json \
  --base-guide cargo-guide.md \
  --project my-gcp-project \
  -o AGENTIC_AI_CARGO_GUIDE.md

# Standalone mode: enhancement sections only
wisdomguard api-ir.json --project my-gcp-project

# Dry run: print what would be sent to the LLM
wisdomguard cargo-ir.json --dry-run

# Use a specific Gemini model and region
wisdomguard ir.json \
  --model gemini-2.5-pro \
  --location europe-west1 \
  --project my-project

# JSON output: raw enhancement data only
wisdomguard ir.json --output-format json -o enhancements.json

# Using environment variables instead of flags
export GOOGLE_CLOUD_PROJECT=my-project
export VERTEX_AI_LOCATION=us-central1
wisdomguard ir.json --base-guide guide.md -o enhanced.md
```

## Modes

### Merge Mode (`--base-guide`)

The recommended approach. WisdomGuard inserts the four enhancement sections at specific positions within the existing base guide:

- **Common Workflows** — inserted before the first `##` heading (after Quick Reference)
- **Common Mistakes** and **Error Messages** — inserted before the `---` footer line
- **Key Commands** — included in the standalone output; in merge mode the base guide's Quick Reference already covers this

The base guide's structure (Quick Reference, Command/Endpoint Reference, Global Options, footer) is preserved intact.

### Standalone Mode

Without `--base-guide`, WisdomGuard produces a guide containing only the four enhancement sections. Useful for appending to a base guide manually or for inspecting what was generated.

## LLM Behaviour

| Parameter | Value |
|-----------|-------|
| Model | `gemini-2.5-flash` (default) |
| Temperature | 0.2 (low, for consistency) |
| Max output tokens | 8,192 |
| Request timeout | 60 seconds |
| Retry attempts | 3 (on HTTP 429 and 5xx) |
| Retry backoff | 1s → 2s → 4s (exponential) |
| Response size limit | 1 MB |

On API failure after all retries, or on LLM response parse failure, WisdomGuard falls back to empty enhancement sections and exits with code `1`.

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error (API failure, parse error, bad arguments) |
| `2` | File/security error (symlink, path traversal, system directory) |
| `3` | Authentication error (invalid credentials, permission denied) |

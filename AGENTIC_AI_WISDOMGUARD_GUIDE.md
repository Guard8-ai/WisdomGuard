# wisdomguard for AI Agents

Enhance agentic AI guides with VertexAI Gemini insights

## Quick Reference

```bash
# Enhance a CliGuard JSON IR
wisdomguard cargo_ir.json --project my-gcp-project

# Enhance an ApiGuard JSON IR
wisdomguard api_ir.json --project my-gcp-project

# Write output to a file
wisdomguard ir.json --project my-gcp-project --output guide.md

# Merge enhancements into an existing Markdown guide
wisdomguard ir.json --base-guide base-guide.md --project my-gcp-project --output enhanced.md

# Use a specific Gemini model
wisdomguard ir.json --project my-gcp-project --model gemini-2.5-pro

# Use a specific region
wisdomguard ir.json --project my-gcp-project --location europe-west1

# Dry run — print the prompt without calling the API
wisdomguard ir.json --project my-gcp-project --dry-run

# Output raw JSON enhancements
wisdomguard ir.json --project my-gcp-project --output-format json

# Read IR from stdin (pipe from cliguard/apiguard)
cliguard cargo --format json | wisdomguard /dev/stdin --project my-gcp-project
apiguard openapi.yaml --output-format json | wisdomguard /dev/stdin --project my-gcp-project
```

## Global Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--base-guide` | string | - | Base guide markdown to merge enhancements into (from cliguard, apiguard, or docguard) |
| `--model` | string | gemini-2.5-flash | Gemini model to use |
| `--project` | string | `$GOOGLE_CLOUD_PROJECT` | Google Cloud project ID |
| `--location` | string | `$VERTEX_AI_LOCATION` or `us-central1` | Vertex AI region |
| `-o, --output` | string | stdout | Output file path |
| `--output-format` | string | md | Output format: md or json [possible values: md, json] |
| `--dry-run` | bool | false | Print the prompt without making an API call |

---

## Common Workflows

### Enhance a CliGuard guide and write to file

```bash
# Generate IR from cargo
cliguard cargo --format json --output cargo_ir.json

# Enhance with WisdomGuard
wisdomguard cargo_ir.json \
  --project my-gcp-project \
  --output docs/cargo-guide.md

# The output now contains:
# - Common Workflows (multi-step bash recipes)
# - Common Mistakes (wrong/right/why table)
# - Key Commands (top 20% covering 80% of use cases)
# - Error Messages (error string → solution table)
```

### Enhance an ApiGuard guide

```bash
apiguard openapi.yaml --output-format json --output api_ir.json

wisdomguard api_ir.json \
  --project my-gcp-project \
  --output docs/api-guide.md
```

### Merge enhancements into an existing base guide

```bash
# Generate the base guide from CliGuard
cliguard kubectl --output kubectl-base.md

# Generate the IR
cliguard kubectl --format json --output kubectl_ir.json

# Merge — enhancements are injected at the correct positions
wisdomguard kubectl_ir.json \
  --base-guide kubectl-base.md \
  --project my-gcp-project \
  --output kubectl-enhanced.md
```

### Inspect the prompt before spending API credits

```bash
# Print the full Gemini prompt without making an API call
wisdomguard cargo_ir.json \
  --project my-gcp-project \
  --dry-run
```

### Use environment variables to avoid repeating flags

```bash
export GOOGLE_CLOUD_PROJECT=my-gcp-project
export VERTEX_AI_LOCATION=europe-west1

wisdomguard cargo_ir.json --output cargo-guide.md
wisdomguard api_ir.json --output api-guide.md
```

### Get structured JSON output for custom rendering

```bash
wisdomguard cargo_ir.json \
  --project my-gcp-project \
  --output-format json \
  --output enhancements.json

# Extract just the workflows
jq '.workflows[] | .title' enhancements.json

# Count gotchas
jq '.gotchas | length' enhancements.json
```

### Full pipeline in a single command

```bash
# CLI tool
cliguard docker --format json \
  | wisdomguard /dev/stdin --project my-gcp-project --output docker-guide.md

# API spec
apiguard openapi.yaml --output-format json \
  | wisdomguard /dev/stdin --project my-gcp-project --output api-guide.md
```

---

## Common Mistakes

| Wrong | Right | Why |
|-------|-------|-----|
| Running `wisdomguard` without `--project` and without `GOOGLE_CLOUD_PROJECT` set | `wisdomguard ir.json --project my-gcp-project` | WisdomGuard has no way to route the Vertex AI request without a project; exits with code 3 |
| Passing a Markdown guide directly to wisdomguard: `wisdomguard guide.md` | Pass the JSON IR: `wisdomguard ir.json` | WisdomGuard reads the IR to understand tool structure; passing Markdown produces an empty or wrong output |
| `wisdomguard ir.json --output guide.md` without `--project` in a CI pipeline | Set `GOOGLE_CLOUD_PROJECT` as a CI env var | `--project` is required or the env var must be set; the call will fail with exit code 3 |
| Skipping `--dry-run` before a large batch run | Always dry-run first: `wisdomguard ir.json --project … --dry-run` | Verifies authentication and prompt quality before spending API quota on dozens of IRs |
| Using `--model gemini-1.0-pro` for large tool IRs | Use default `gemini-2.5-flash` or `--model gemini-2.5-pro` | Older models have smaller context windows; large IRs (e.g. full gcloud) may exceed the limit |
| Passing a CliGuard IR to `--base-guide` instead of `--base-guide` pointing to the Markdown file | `--base-guide kubectl-guide.md` (the `.md` file, not the `.json` IR) | `--base-guide` expects Markdown output from a guard tool, not the JSON IR |

---

## Key Commands

- `wisdomguard <ir.json> --project <id>` — enhance IR, output enhanced Markdown to stdout
- `wisdomguard <ir.json> --project <id> --output guide.md` — write to file
- `wisdomguard <ir.json> --base-guide base.md --project <id> --output enhanced.md` — merge mode
- `wisdomguard <ir.json> --project <id> --dry-run` — inspect prompt, no API call
- `wisdomguard <ir.json> --project <id> --output-format json` — raw `EnhancementResponse` JSON
- `wisdomguard /dev/stdin --project <id>` — read IR from stdin pipe

---

## Error Messages

| Error / Exit Code | Meaning | Solution |
|-------------------|---------|---------|
| Exit code `1` | I/O error (bad input path or output path) | Check the IR file exists and the output directory is writable |
| Exit code `2` | IR parse error — file is not valid JSON or not a recognised IR schema | Run `jq . ir.json` to validate JSON; confirm the IR was produced by ApiGuard or CliGuard |
| Exit code `3` | Authentication or GCP configuration error | Run `gcloud auth application-default print-access-token`; set `--project` or `GOOGLE_CLOUD_PROJECT`; confirm `roles/aiplatform.user` is granted |
| Exit code `4` | Gemini API error (quota, model unavailable, network) | Check VertexAI quota in GCP Console; try `--location us-east4`; retry after a pause |
| `Authentication failed` in error message | ADC credentials missing or expired | Run `gcloud auth application-default login` or set `GOOGLE_APPLICATION_CREDENTIALS` |
| `Project not found` | GCP project ID is wrong or Vertex AI API is not enabled | Verify with `gcloud projects describe <id>`; enable with `gcloud services enable aiplatform.googleapis.com` |
| `Model not found: …` | Model ID does not exist in the selected region | Use `gemini-2.5-flash` (default) or `gemini-2.5-pro`; check availability for your `--location` |
| Output Markdown contains no enhancement sections | IR was parsed but had no commands/endpoints to calibrate the prompt | Confirm the IR has a non-empty `commands` or `endpoints` array: `jq '.commands | length' ir.json` |

---
**Framework**: clap | **Version**: wisdomguard 0.1.0

# WisdomGuard in the Guard8.ai Pipeline

WisdomGuard is the enhancement stage of the Guard8.ai tool chain. It takes a JSON IR produced by ApiGuard or CliGuard and adds LLM-generated insights that make the guide genuinely useful for AI agents and developers.

---

## Pipeline Overview

```
ApiGuard or CliGuard      WisdomGuard             Output
JSON IR (ApiSpec or  ──────────────────►  Enhanced Markdown Guide
 ToolSpec)                                  or JSON EnhancementResponse
```

WisdomGuard makes a **single Gemini API call** and generates four enhancement sections in one shot. It then either:
- Outputs a standalone enhanced guide (no `--base-guide`)
- Merges enhancements into an existing Markdown guide (`--base-guide`)

---

## Full Pipeline Example

### CLI tool (CliGuard → WisdomGuard)

```bash
# Step 1: Generate JSON IR from the binary
cliguard cargo --format json --output cargo_ir.json

# Step 2: Enhance with WisdomGuard
wisdomguard cargo_ir.json \
  --project my-gcp-project \
  --output cargo-guide.md
```

### API spec (ApiGuard → WisdomGuard)

```bash
# Step 1: Parse the OpenAPI spec
apiguard openapi.yaml --output-format json --output api_ir.json

# Step 2: Enhance with WisdomGuard
wisdomguard api_ir.json \
  --project my-gcp-project \
  --output api-guide.md
```

### One-liner pipes

```bash
# CLI tool — pipe directly
cliguard docker --format json | wisdomguard /dev/stdin --project my-gcp-project

# API spec — pipe directly
apiguard openapi.yaml --output-format json | wisdomguard /dev/stdin --project my-gcp-project
```

---

## Merge Mode

When `--base-guide` is provided, WisdomGuard injects enhancements at specific positions in the existing guide rather than generating a new file:

```bash
# Generate base guide from CliGuard
cliguard kubectl --output kubectl-base.md

# Generate IR separately
cliguard kubectl --format json --output kubectl_ir.json

# Merge enhancements into base guide
wisdomguard kubectl_ir.json \
  --base-guide kubectl-base.md \
  --project my-gcp-project \
  --output kubectl-enhanced.md
```

Insertion positions in merge mode:
```
[Quick Reference]           ← kept intact
                            ← Common Workflows inserted here
[Command/Endpoint Reference]← kept intact
[Global Options]            ← kept intact
                            ← Common Mistakes inserted here
                            ← Error Messages inserted here
---                         ← footer separator
**Format**: ...             ← kept intact
```

Note: **Key Commands** is omitted in merge mode — the base guide's Quick Reference already serves this purpose.

---

## IR Type Auto-Detection

WisdomGuard detects the IR type from the JSON fields and adapts the Gemini prompt accordingly:

| IR Type | Detection | LLM Prompt Adapts |
|---------|-----------|-------------------|
| `cli` | `framework` + `commands` fields | Generates bash workflows with subcommands/flags |
| `api` | `endpoints` + `spec_format` fields | Generates curl workflows, HTTP error solutions |
| `doc` | `modules` + `source_format` fields | Generates library usage patterns |

---

## Inspecting the Prompt

Before making an API call, inspect the exact prompt WisdomGuard will send:

```bash
wisdomguard cargo_ir.json \
  --project my-gcp-project \
  --dry-run
```

This prints the complete prompt to stdout and exits — no API call, no cost.

---

## JSON Output

To get the raw enhancements as structured data (useful for custom rendering or post-processing):

```bash
wisdomguard cargo_ir.json \
  --project my-gcp-project \
  --output-format json \
  --output enhancements.json
```

```json
{
  "workflows": [{ "title": "...", "steps": "..." }],
  "gotchas": [{ "wrong": "...", "right": "...", "why": "..." }],
  "key_items": ["..."],
  "error_solutions": [{ "error": "...", "solution": "..." }]
}
```

---

## CI / Automation Example

```yaml
# .github/workflows/docs.yml
- name: Generate enhanced guides
  run: |
    # CLI guide
    cliguard cargo --format json --output cargo_ir.json
    wisdomguard cargo_ir.json \
      --project ${{ vars.GCP_PROJECT }} \
      --output docs/cargo-guide.md

    # API guide
    apiguard openapi.yaml --output-format json --output api_ir.json
    wisdomguard api_ir.json \
      --project ${{ vars.GCP_PROJECT }} \
      --output docs/api-guide.md
  env:
    GOOGLE_APPLICATION_CREDENTIALS: ${{ secrets.GCP_SA_KEY_PATH }}
    VERTEX_AI_LOCATION: us-central1
```

---

## Related

- [Enhancements](enhancements.md) — what each section contains and how it is generated
- [Authentication](authentication.md) — GCP credentials setup
- [Usage](usage.md) — complete CLI reference
- [ApiGuard docs](../ApiGuard/docs/index.md) — upstream IR source for API specs
- [CliGuard docs](../CliGuard/docs/index.md) — upstream IR source for CLI tools

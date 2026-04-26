# WisdomGuard

**Enhance agentic AI guides with Google VertexAI Gemini insights.**

WisdomGuard takes the JSON IR produced by ApiGuard, CliGuard, or DocGuard and calls the Gemini API to add four high-value sections: multi-step workflows, wrong-vs-right gotchas, prioritized key commands, and error solutions.

---

## What It Adds

| Section | Description |
|---------|-------------|
| **Common Workflows** | 3–7 named multi-step bash recipes for real tasks |
| **Common Mistakes** | Wrong / Right / Why table of the most frequent errors |
| **Key Commands** | Top 20% of commands/endpoints that cover 80% of use cases |
| **Error Messages** | Common errors with actionable solutions |

---

## Quick Start

```bash
# Install
cargo install --path .

# Authenticate with GCP
gcloud auth application-default login

# Enhance an existing guide (merge mode — recommended)
wisdomguard ir.json --base-guide guide.md --project my-gcp-project -o AGENTIC_AI_GUIDE.md

# Standalone enhancement only
wisdomguard ir.json --project my-gcp-project

# Preview prompts without calling the API
wisdomguard ir.json --dry-run
```

---

## Navigation

- [Installation](installation.md)
- [Usage & CLI Reference](usage.md)
- [Authentication](authentication.md)
- [Enhancement Sections](enhancements.md)
- [Pipeline Integration](pipeline.md)

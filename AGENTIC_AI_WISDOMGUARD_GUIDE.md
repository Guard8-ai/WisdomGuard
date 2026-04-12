# wisdomguard for AI Agents

Enhance agentic AI guides with VertexAI Gemini insights

## Quick Reference

```bash
# Global options
wisdomguard --base-guide <value>            # Base guide markdown to merge enhancements into ...
wisdomguard --model <value>                 # Gemini model to use [default: gemini-2.5-flash]
wisdomguard --project <value>               # Google Cloud project ID (or set 'GOOGLE_CLOUD_P...
wisdomguard --location <value>              # Vertex AI location (or set 'VERTEX_AI_LOCATION')
wisdomguard --output <value>                # Output file path (stdout if not specified)
wisdomguard --output-format <value>         # Output format: md or json [default: md] [possib...
wisdomguard --dry-run                       # Show prompts without calling the API
```

## Global Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--base-guide` | string | - | Base guide markdown to merge enhancements into (from cliguard, apiguard, or docguard) |
| `--model` | string | gemini-2.5-flash | Gemini model to use [default: gemini-2.5-flash] |
| `--project` | string | - | Google Cloud project ID (or set 'GOOGLE_CLOUD_PROJECT') |
| `--location` | string | - | Vertex AI location (or set 'VERTEX_AI_LOCATION') |
| `-o, --output` | string | - | Output file path (stdout if not specified) |
| `--output-format` | string | md | Output format: md or json [default: md] [possible values: md, json] |
| `--dry-run` | bool | - | Show prompts without calling the API |

---
**Framework**: clap | **Version**: wisdomguard 0.1.0

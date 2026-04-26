# Authentication

WisdomGuard calls the Google VertexAI Gemini API. Authentication is handled via Google Cloud's Application Default Credentials (ADC).

---

## Option 1: User Credentials (Development)

The simplest setup for local development.

```bash
# Install gcloud CLI if not already installed
# https://cloud.google.com/sdk/docs/install

# Login with your Google account
gcloud auth application-default login

# Set your default project
gcloud config set project my-gcp-project
```

After this, WisdomGuard will find credentials automatically. No additional configuration needed.

---

## Option 2: Service Account (CI/CD and Production)

For non-interactive environments (CI pipelines, servers, containers).

```bash
# Create a service account
gcloud iam service-accounts create wisdomguard-sa \
  --display-name "WisdomGuard Service Account"

# Grant Vertex AI user role
gcloud projects add-iam-policy-binding my-gcp-project \
  --member "serviceAccount:wisdomguard-sa@my-gcp-project.iam.gserviceaccount.com" \
  --role "roles/aiplatform.user"

# Create and download a key
gcloud iam service-accounts keys create wisdomguard-key.json \
  --iam-account wisdomguard-sa@my-gcp-project.iam.gserviceaccount.com

# Point ADC at the key file
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/wisdomguard-key.json
```

---

## Option 3: Workload Identity (GKE / Cloud Run)

If WisdomGuard runs on Google Cloud infrastructure, attach the service account to the workload and ADC resolves automatically — no key file needed.

---

## Required IAM Role

WisdomGuard needs the **Vertex AI User** role (`roles/aiplatform.user`) on the GCP project. This role grants permission to call `aiplatform.googleapis.com/generateContent`.

To verify:
```bash
gcloud projects get-iam-policy my-gcp-project \
  --flatten="bindings[].members" \
  --filter="bindings.role:roles/aiplatform.user"
```

---

## Enabling the Vertex AI API

```bash
gcloud services enable aiplatform.googleapis.com --project my-gcp-project
```

---

## Verifying Authentication

```bash
# Check ADC is configured
gcloud auth application-default print-access-token

# Test with dry run (no API call, just shows the prompt)
wisdomguard ir.json --dry-run --project my-gcp-project

# Test a real call
wisdomguard ir.json --project my-gcp-project
```

---

## Exit Code 3

If WisdomGuard exits with code `3`, the error message contains "Authentication" or "credentials". Check:

1. `gcloud auth application-default print-access-token` — should return a token, not an error
2. The service account has `roles/aiplatform.user` on the project
3. `GOOGLE_CLOUD_PROJECT` is set or `--project` is specified
4. The Vertex AI API is enabled on the project

---

## Region Selection

WisdomGuard defaults to `us-central1`. Change with `--location` or `VERTEX_AI_LOCATION`:

```bash
wisdomguard ir.json --location europe-west1 --project my-project
```

Available regions: `us-central1`, `us-east4`, `us-west1`, `europe-west1`, `europe-west4`, `asia-northeast1`, and others — see [VertexAI locations](https://cloud.google.com/vertex-ai/docs/general/locations).

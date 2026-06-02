# Logging Guidelines

`nako-media-probe` currently has no logging. If diagnostics are added, they
must help operators understand probe execution without leaking source secrets.

## Required Patterns

- Prefer caller-level structured events for scan/probe orchestration.
- Log provider name, stream counts, and high-level outcome when useful.
- Keep local paths and storage URIs redacted or policy-controlled in logs.
- Keep stderr content out of routine logs; return it through provider errors
  where callers can classify it.

## Suggested Fields

| Field | Purpose |
|-------|---------|
| `provider` | Probe backend, currently `ffprobe` |
| `source_scheme` | Storage scheme without full locator |
| `stream_count` | Number of streams parsed |
| `duration_ms` | Optional parsed container duration |
| `outcome` | `success`, `unsupported`, or `provider_error` |

## Forbidden Patterns

- Do not log full local file paths, signed URLs, credentials, source locators,
  or raw ffprobe JSON by default.
- Do not log ffprobe stderr at info level.
- Do not add per-stream verbose logs in hot scan paths unless gated by tracing
  level.
- Do not initialize global logging from this crate.

## Review Checklist

- Would the log be safe in a multi-user server?
- Is the diagnostic better emitted by `nako-library` where source context and
  failure persistence are available?
- Does the log avoid high-cardinality raw paths and provider blobs?

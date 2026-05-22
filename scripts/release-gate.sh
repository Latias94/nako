#!/usr/bin/env bash
set -euo pipefail

mode="fast"
postgres_url="${NAKO_TEST_POSTGRES_URL:-}"
skip_redaction_inventory="false"
redaction_inventory_pattern='storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|artifact_root|local_path|database_url|token|secret'

usage() {
  cat <<'USAGE'
Usage: scripts/release-gate.sh [--mode docs|fast|db|api|postgres|container|workspace|all] [--postgres-url URL] [--skip-redaction-inventory]

Runs Nako's local release gate without deleting user data or assuming Docker.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      mode="${2:?missing value for --mode}"
      shift 2
      ;;
    --postgres-url)
      postgres_url="${2:?missing value for --postgres-url}"
      shift 2
      ;;
    --skip-redaction-inventory)
      skip_redaction_inventory="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 64
      ;;
  esac
done

case "$mode" in
  docs|fast|db|api|postgres|container|workspace|all) ;;
  *)
    echo "Invalid mode: $mode" >&2
    usage >&2
    exit 64
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

release_gate_output="target/release-gate"
redaction_inventory_path="$release_gate_output/redaction-inventory.txt"

step() {
  echo
  echo "==> $*"
  "$@"
}

contains_mode() {
  local candidate="$1"
  [[ "$mode" == "$candidate" || "$mode" == "all" ]]
}

inventory() {
  echo
  echo "==> redaction inventory scan"
  mkdir -p "$release_gate_output"

  if rg -n "$redaction_inventory_pattern" crates docs >"$redaction_inventory_path" 2>&1; then
    local match_count
    match_count="$(wc -l <"$redaction_inventory_path" | tr -d '[:space:]')"
    echo "$match_count matches written to $redaction_inventory_path."
    return 0
  fi

  local exit_code=$?
  if [[ "$exit_code" -eq 1 ]]; then
    : >"$redaction_inventory_path"
    echo "No matches. Inventory written to $redaction_inventory_path."
    return 0
  fi

  cat "$redaction_inventory_path" >&2
  return "$exit_code"
}

api_sdk_gate() {
  step cargo check -p nako-api --tests
  step cargo check -p nako-client --tests
  step cargo check -p nako-client-protocol --tests
  step cargo nextest run -p nako-api openapi --no-fail-fast
  step cargo nextest run -p nako-api sdk --no-fail-fast
  step cargo nextest run -p nako-api admin_contract --no-fail-fast
  step cargo nextest run -p nako-client --no-fail-fast
  step cargo nextest run -p nako-client-protocol --no-fail-fast
  step cargo tree -p nako-client
  step cargo tree -p nako-client-protocol
  step npm run generate --prefix sdk/typescript
  step npm run check --prefix sdk/typescript
  step npm run generate:admin-api --prefix apps/admin-web
  step npm run check --prefix apps/admin-web
  step git diff --check
}

container_gate() {
  local media_root="$repo_root/target/release-gate/media/movies"
  mkdir -p "$media_root"

  step cargo nextest run -p nako-server config --no-fail-fast

  echo
  echo "==> docker compose config for Nako SQLite stack"
  NAKO_ADMIN_TOKEN="release-gate-admin-token" \
    NAKO_MEDIA_ROOT="$media_root" \
    docker compose -f deploy/compose/nako-sqlite.yml config >/dev/null

  echo
  echo "==> docker compose config for Nako PostgreSQL stack"
  NAKO_ADMIN_TOKEN="release-gate-admin-token" \
    NAKO_POSTGRES_PASSWORD="release-gate-postgres-password" \
    NAKO_MEDIA_ROOT="$media_root" \
    docker compose -f deploy/compose/nako-postgres.yml config >/dev/null
}

echo "Nako release gate"
echo "Mode: $mode"
echo "Repository: $repo_root"

step cargo fmt --all -- --check
step git diff --check

if [[ "$skip_redaction_inventory" != "true" ]]; then
  inventory
fi

if [[ "$mode" == "docs" ]]; then
  echo
  echo "Docs-safe release gate completed."
  exit 0
fi

if contains_mode "fast" || contains_mode "db"; then
  step cargo check -p nako-db --tests
  step cargo nextest run -p nako-db sqlite_managed_artwork_contract --no-fail-fast
fi

if contains_mode "fast" || contains_mode "api"; then
  step cargo check -p nako-server --tests
  api_sdk_gate
  step cargo nextest run -p nako-api managed_artwork --no-fail-fast
  step cargo nextest run -p nako-server managed_artwork --no-fail-fast
  step cargo nextest run -p nako-server self_host_smoke --no-fail-fast
fi

if contains_mode "container"; then
  container_gate
fi

if contains_mode "workspace"; then
  step cargo check --workspace --tests
  step cargo nextest run --workspace --no-fail-fast
fi

if contains_mode "postgres"; then
  if [[ -z "$postgres_url" ]]; then
    step bash scripts/postgres-contract-harness.sh --suite managed-artwork
  else
    step bash scripts/postgres-contract-harness.sh --suite managed-artwork --database-url "$postgres_url"
  fi
fi

echo
echo "Nako release gate completed."

#!/usr/bin/env bash
set -euo pipefail

backend="sqlite"
postgres_contracts_only="false"

usage() {
  cat <<'USAGE'
Usage: scripts/self-host-smoke.sh [--backend sqlite|postgres] [--postgres-contracts-only]
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --backend)
      backend="${2:?missing value for --backend}"
      shift 2
      ;;
    --postgres-contracts-only)
      postgres_contracts_only="true"
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

case "$backend" in
  sqlite|postgres) ;;
  *)
    echo "Invalid backend: $backend" >&2
    usage >&2
    exit 64
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

step() {
  echo
  echo "==> $*"
  "$@"
}

if [[ "$backend" == "postgres" || "$postgres_contracts_only" == "true" ]]; then
  step bash scripts/postgres-contract-harness.sh --suite managed-artwork
  exit 0
fi

step cargo nextest run -p taru-server self_host_smoke --no-fail-fast

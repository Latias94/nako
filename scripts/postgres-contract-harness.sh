#!/usr/bin/env bash
set -euo pipefail

suite="managed-artwork"
database_url="${NAKO_TEST_POSTGRES_URL:-}"
port="55432"
keep_data="false"
require_tooling="false"

storage_runtime_filters=("postgres_storage_backend_health_contract" "postgres_vfs_staging_contract")
source_identity_filters=(
  "postgres_library_media_contract_preserves_library_scoped_source_identity"
  "postgres_scan_commit_contract_writes_full_source_unit_and_resolves_failure"
  "postgres_source_duplicate_contract"
  "postgres_vfs_staging_contract_round_trips_attribution_variants"
  "postgres_vfs_staging_contract_preserves_reservation_budget_and_leases"
)
job_runtime_filters=(
  "postgres_job_lease_contract_claims_next_with_worker_token_and_filter"
  "postgres_job_lease_contract_heartbeats_and_completes_with_run_token_fence"
  "postgres_job_lease_contract_cancel_requests_are_durable_and_acknowledged_by_owner"
  "postgres_job_lease_contract_recovers_only_expired_running_leases"
  "postgres_job_retry_contract_persists_backoff_and_redacted_queue_pressure"
  "postgres_job_retry_contract_priority_policy_orders_fairly_and_recovers"
)

usage() {
  cat <<'USAGE'
Usage: scripts/postgres-contract-harness.sh [--suite managed-artwork|storage-runtime|source-identity|job-runtime|storage-source-parity|all-contracts] [--database-url URL] [--port PORT] [--keep-data] [--require-tooling]

Runs Nako's ignored PostgreSQL contract tests. If a database URL is supplied,
the harness uses it directly. Without a URL, it starts a temporary local
PostgreSQL cluster under target/postgres-contract when initdb/pg_ctl/createdb
are available; otherwise it prints a clear skip and exits successfully unless
--require-tooling is set.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --suite)
      suite="${2:?missing value for --suite}"
      shift 2
      ;;
    --database-url)
      database_url="${2:?missing value for --database-url}"
      shift 2
      ;;
    --port)
      port="${2:?missing value for --port}"
      shift 2
      ;;
    --keep-data)
      keep_data="true"
      shift
      ;;
    --require-tooling)
      require_tooling="true"
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

case "$suite" in
  managed-artwork) test_filters=("postgres_managed_artwork_contract") ;;
  storage-runtime) test_filters=("${storage_runtime_filters[@]}") ;;
  source-identity)
    test_filters=("${source_identity_filters[@]}")
    ;;
  job-runtime)
    test_filters=("${job_runtime_filters[@]}")
    ;;
  storage-source-parity) test_filters=() ;;
  all-contracts) test_filters=("postgres_") ;;
  *)
    echo "Invalid suite: $suite" >&2
    usage >&2
    exit 64
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

harness_root="target/postgres-contract"
data_dir="$harness_root/data"
log_path="$harness_root/postgres.log"
database_name="nako_contract"
user_name="nako"
started_local_server="false"
local_postgres_stopped="true"

step() {
  echo
  echo "==> $*"
  "$@"
}

run_nextest() {
  local -a filters=("$@")
  step cargo nextest run -p nako-db "${filters[@]}" --run-ignored ignored-only --no-fail-fast
}

cleanup_data() {
  if [[ "$keep_data" == "true" ]]; then
    echo
    echo "Keeping PostgreSQL harness data at $harness_root."
    return 0
  fi

  if [[ "$started_local_server" == "true" && "$local_postgres_stopped" != "true" ]]; then
    echo "WARNING: Keeping PostgreSQL harness data at $harness_root because the local server did not confirm shutdown." >&2
    return 0
  fi

  if [[ -e "$harness_root" ]]; then
    case "$(cd "$harness_root/.." && pwd -P)/$(basename "$harness_root")" in
      "$(cd target && pwd -P)"/*) rm -rf "$harness_root" ;;
      *)
        echo "Refusing to remove PostgreSQL harness data outside target/: $harness_root" >&2
        return 1
        ;;
    esac
  fi
}

stop_postgres() {
  if [[ "$started_local_server" == "true" ]]; then
    if pg_ctl stop -D "$data_dir" -m fast -w -t 90; then
      local_postgres_stopped="true"
    else
      local_postgres_stopped="false"
      echo "WARNING: Failed to stop local PostgreSQL cleanly." >&2
    fi
  fi
}

trap 'stop_postgres; cleanup_data' EXIT

mkdir -p "$harness_root"

if [[ -z "$database_url" ]]; then
  missing_tools=()
  for tool in initdb pg_ctl createdb; do
    if ! command -v "$tool" >/dev/null 2>&1; then
      missing_tools+=("$tool")
    fi
  done

  if [[ "${#missing_tools[@]}" -gt 0 ]]; then
    message="Skipping PostgreSQL contract harness because NAKO_TEST_POSTGRES_URL was not provided and local PostgreSQL tooling is missing: ${missing_tools[*]}."
    if [[ "$require_tooling" == "true" ]]; then
      echo "$message" >&2
      exit 1
    fi
    echo "WARNING: $message" >&2
    exit 0
  fi

  cleanup_data
  mkdir -p "$harness_root"

  step initdb -D "$data_dir" -U "$user_name" -A trust -E UTF8 --no-locale
  step pg_ctl start -D "$data_dir" -l "$log_path" -w -t 60 -o "-p $port -h 127.0.0.1"
  started_local_server="true"
  local_postgres_stopped="false"

  step createdb -h 127.0.0.1 -p "$port" -U "$user_name" "$database_name"
  database_url="postgres://$user_name@127.0.0.1:$port/$database_name"
else
  echo "Using caller-provided PostgreSQL database URL."
fi

export NAKO_TEST_POSTGRES_URL="$database_url"
if [[ "$suite" == "storage-source-parity" ]]; then
  run_nextest "${storage_runtime_filters[@]}"
  run_nextest "${source_identity_filters[@]}"
else
  run_nextest "${test_filters[@]}"
fi

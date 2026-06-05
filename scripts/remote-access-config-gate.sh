#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

output_dir="${1:-target/release-gate/remote-access}"

python_bin="${PYTHON:-}"
if [[ -z "$python_bin" ]]; then
  if command -v python3 >/dev/null 2>&1; then
    python_bin="python3"
  elif command -v python >/dev/null 2>&1; then
    python_bin="python"
  else
    echo "python3 or python is required to validate config-check JSON." >&2
    exit 127
  fi
fi

validate_report() {
  local report_path="$1"
  local fixture_name="$2"
  local expected_checks="$3"

  "$python_bin" - "$report_path" "$fixture_name" "$expected_checks" <<'PY'
import json
import sys

report_path, fixture_name, expected_checks = sys.argv[1:4]
with open(report_path, "r", encoding="utf-8") as handle:
    report = json.load(handle)

if report.get("status") != "pass":
    raise SystemExit(f"{fixture_name} expected pass status, got {report.get('status')}")

checks = {check.get("id"): check.get("status") for check in report.get("checks", [])}
for check_id in expected_checks.split(","):
    if check_id not in checks:
        raise SystemExit(f"{fixture_name} missing expected check {check_id}")
    if checks[check_id] != "pass":
        raise SystemExit(f"{fixture_name} expected {check_id} to pass, got {checks[check_id]}")
PY
}

assert_absent() {
  local report_path="$1"
  local fixture_name="$2"
  local value="$3"

  if grep -qiF -- "$value" "$report_path"; then
    echo "$fixture_name leaked sensitive fixture value: $value" >&2
    return 1
  fi
}

run_fixture() {
  local fixture_name="$1"
  local config_path="$2"
  shift 2
  local output_path="$output_dir/${fixture_name}-config-check.json"
  local temp_output_path="$output_path.tmp"

  echo
  echo "==> remote access config-check: $fixture_name"
  mkdir -p "$output_dir"

  if ! NAKO_ADMIN_TOKEN="remote-access-fixture-admin-token" \
    NAKO_REMOTE_ACCESS_GATE_TUNNEL_TOKEN="remote-access-fixture-tunnel-token" \
    cargo run -q -p nako-server -- --config "$config_path" config-check --json --create-dirs >"$temp_output_path"; then
    rm -f "$temp_output_path"
    echo "$fixture_name config-check failed." >&2
    return 1
  fi

  mv "$temp_output_path" "$output_path"

  validate_report \
    "$output_path" \
    "$fixture_name" \
    "network.access,network.proxy,network.origins,network.tunnel_providers"

  for sensitive_value in "$@"; do
    assert_absent "$output_path" "$fixture_name" "$sensitive_value"
  done

  echo "Report: $output_path"
}

echo "Nako remote access config gate"
echo "Repository: $repo_root"
echo "Output: $output_dir"

run_fixture \
  "reverse-proxy" \
  "deploy/remote-access/reverse-proxy.nako.toml" \
  "127.0.0.1" \
  "nako-reverse.redaction.invalid" \
  "player-reverse.redaction.invalid" \
  "webdav-reverse.redaction.invalid" \
  "reverse-url-token" \
  "10.66.10.5" \
  "remote-access-fixture-admin-token" \
  "x-forwarded-host" \
  "x-forwarded-proto"

run_fixture \
  "tunnel-provider" \
  "deploy/remote-access/tunnel-provider.nako.toml" \
  "127.0.0.1" \
  "nako-tunnel.redaction.invalid" \
  "player-tunnel.redaction.invalid" \
  "cloudflare-tunnel.redaction.invalid" \
  "webdav-tunnel.redaction.invalid" \
  "tunnel-url-token" \
  "tunnel-library-token" \
  "remote-access-fixture-admin-token" \
  "remote-access-fixture-tunnel-token" \
  "x-forwarded-host" \
  "x-forwarded-proto"

echo
echo "Remote access config gate completed."

#!/usr/bin/env bash
set -euo pipefail

output_dir="target/package-release"
version=""
skip_build="false"
dry_run="false"

usage() {
  cat <<'USAGE'
Usage: scripts/package-release.sh [--output-dir DIR] [--version VERSION] [--skip-build] [--dry-run]

Builds and packages the nako-server release artifact with a manifest and
SHA256SUMS. The script does not publish artifacts or push container images.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      output_dir="${2:?missing value for --output-dir}"
      shift 2
      ;;
    --version)
      version="${2:?missing value for --version}"
      shift 2
      ;;
    --skip-build)
      skip_build="true"
      shift
      ;;
    --dry-run)
      dry_run="true"
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

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

step() {
  echo
  echo "==> $*"
  "$@"
}

workspace_version() {
  if [[ -n "$version" ]]; then
    printf '%s\n' "$version"
    return
  fi
  awk -F '"' '/^[[:space:]]*version[[:space:]]*=/ { print $2; exit }' Cargo.toml
}

host_target_triple() {
  if command -v rustc >/dev/null 2>&1; then
    rustc -vV | awk '/^host:/ { print $2; exit }'
  else
    printf 'unknown-target\n'
  fi
}

git_revision() {
  git rev-parse HEAD 2>/dev/null || printf 'unknown\n'
}

git_dirty() {
  if git diff --quiet --ignore-submodules -- && git diff --cached --quiet --ignore-submodules --; then
    printf 'false\n'
  else
    printf 'true\n'
  fi
}

copy_release_path() {
  local source="$1"
  local destination="$2"
  if [[ ! -e "$source" ]]; then
    echo "Required release input does not exist: $source" >&2
    exit 1
  fi
  mkdir -p "$(dirname "$destination")"
  cp -R "$source" "$destination"
}

package_version="$(workspace_version)"
target_triple="$(host_target_triple)"
revision="$(git_revision | tr -d '[:space:]')"
short_revision="${revision:0:12}"
package_id="nako-server-v${package_version}-${target_triple}-${short_revision}"
output_root="$repo_root/$output_dir"
staging_parent="$output_root/staging"
staging_root="$staging_parent/$package_id"
archive_path="$output_root/${package_id}.tar.gz"
manifest_output_path="$output_root/${package_id}.release-manifest.json"
checksums_path="$output_root/SHA256SUMS"
binary_name="nako-server"
binary_path="$repo_root/target/release/$binary_name"
if [[ ! -f "$binary_path" && -f "$binary_path.exe" ]]; then
  binary_name="nako-server.exe"
  binary_path="$repo_root/target/release/$binary_name"
fi

echo "Nako release package"
echo "Package: $package_id"
echo "Output: $output_root"
echo "SkipBuild: $skip_build"

if [[ "$dry_run" == "true" ]]; then
  if ! command -v rustc >/dev/null 2>&1; then
    echo "Dry run note: rustc was not found, so target triple is unknown-target."
  fi
  echo
  echo "Dry run: would build/copy release files, write manifest, archive, and SHA256SUMS."
  exit 0
fi

if ! command -v rustc >/dev/null 2>&1; then
  echo "rustc is required unless --dry-run is used." >&2
  exit 1
fi

if [[ "$skip_build" != "true" ]]; then
  step cargo build --locked --release -p nako-server
fi

if [[ ! -f "$binary_path" ]]; then
  echo "Release binary does not exist: $binary_path" >&2
  exit 1
fi

mkdir -p "$output_root" "$staging_parent"
case "$(python -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$staging_root")" in
  "$(python -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$output_root")"/*) ;;
  *)
    echo "Refusing to operate outside package output directory: $staging_root" >&2
    exit 1
    ;;
esac

rm -rf "$staging_root"
mkdir -p "$staging_root/bin"

copy_release_path "$binary_path" "$staging_root/bin/$binary_name"
copy_release_path "$repo_root/LICENSE" "$staging_root/LICENSE"
copy_release_path "$repo_root/README.md" "$staging_root/README.md"
copy_release_path "$repo_root/Dockerfile" "$staging_root/Dockerfile"
copy_release_path "$repo_root/.dockerignore" "$staging_root/.dockerignore"
copy_release_path "$repo_root/deploy/sqlite/nako.toml" "$staging_root/deploy/sqlite/nako.toml"
copy_release_path "$repo_root/deploy/postgres/nako.toml" "$staging_root/deploy/postgres/nako.toml"
copy_release_path "$repo_root/deploy/container" "$staging_root/deploy/container"
copy_release_path "$repo_root/deploy/compose/.env.example" "$staging_root/deploy/compose/.env.example"
copy_release_path "$repo_root/deploy/compose/nako-sqlite.yml" "$staging_root/deploy/compose/nako-sqlite.yml"
copy_release_path "$repo_root/deploy/compose/nako-postgres.yml" "$staging_root/deploy/compose/nako-postgres.yml"
copy_release_path "$repo_root/docs/deployment/SELF_HOSTED.md" "$staging_root/docs/deployment/SELF_HOSTED.md"
copy_release_path "$repo_root/docs/deployment/RELEASE_ARTIFACTS.md" "$staging_root/docs/deployment/RELEASE_ARTIFACTS.md"
copy_release_path "$repo_root/docs/deployment/BACKUP_RESTORE_UPGRADE.md" "$staging_root/docs/deployment/BACKUP_RESTORE_UPGRADE.md"

manifest_path="$staging_root/release-manifest.json"
find "$staging_root" -type f -printf '%P\n' | sort >"$output_root/included-files.tmp"

PACKAGE_VERSION="$package_version" \
GIT_REVISION="$revision" \
GIT_DIRTY="$(git_dirty)" \
TARGET_TRIPLE="$target_triple" \
ARCHIVE_FILE="$(basename "$archive_path")" \
BINARY_PATH="bin/$binary_name" \
BUILD_COMMAND="$(if [[ "$skip_build" == "true" ]]; then printf 'skipped; existing target/release binary was packaged'; else printf 'cargo build --locked --release -p nako-server'; fi)" \
INCLUDED_FILES_PATH="$output_root/included-files.tmp" \
python - "$manifest_path" <<'PY'
import json
import os
import sys
from datetime import datetime, timezone

manifest_path = sys.argv[1]
with open(os.environ["INCLUDED_FILES_PATH"], encoding="utf-8") as handle:
    included_files = [line.strip().replace("\\", "/") for line in handle if line.strip()]

manifest = {
    "schema_version": 1,
    "package": "nako-server",
    "version": os.environ["PACKAGE_VERSION"],
    "git_revision": os.environ["GIT_REVISION"],
    "git_dirty": os.environ["GIT_DIRTY"] == "true",
    "target_triple": os.environ["TARGET_TRIPLE"],
    "built_at_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
    "archive_file": os.environ["ARCHIVE_FILE"],
    "binary": os.environ["BINARY_PATH"],
    "build_command": os.environ["BUILD_COMMAND"],
    "preflight_command": "nako-server --config /config/nako.toml config-check --create-dirs",
    "included_files": included_files,
}

with open(manifest_path, "w", encoding="utf-8") as handle:
    json.dump(manifest, handle, indent=2, sort_keys=False)
    handle.write("\n")
PY

find "$staging_root" -type f -printf '%P\n' | sort >"$output_root/included-files.tmp"
INCLUDED_FILES_PATH="$output_root/included-files.tmp" python - "$manifest_path" <<'PY'
import json
import os
import sys

manifest_path = sys.argv[1]
with open(manifest_path, encoding="utf-8") as handle:
    manifest = json.load(handle)
with open(os.environ["INCLUDED_FILES_PATH"], encoding="utf-8") as handle:
    manifest["included_files"] = [line.strip().replace("\\", "/") for line in handle if line.strip()]
with open(manifest_path, "w", encoding="utf-8") as handle:
    json.dump(manifest, handle, indent=2, sort_keys=False)
    handle.write("\n")
PY
cp "$manifest_path" "$manifest_output_path"
rm -f "$output_root/included-files.tmp"

rm -f "$archive_path"
tar -C "$staging_parent" -czf "$archive_path" "$package_id"

(
  cd "$output_root"
  sha256sum "$(basename "$archive_path")" "$(basename "$manifest_output_path")" >"$checksums_path"
)

echo
echo "Archive: $archive_path"
echo "Manifest: $manifest_output_path"
echo "Checksums: $checksums_path"

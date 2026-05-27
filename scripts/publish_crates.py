#!/usr/bin/env python3
"""Publish Nako's public crates in dependency order."""

from __future__ import annotations

import argparse
import subprocess
import sys
import time
import tomllib
from dataclasses import dataclass
from pathlib import Path


PUBLIC_CRATES = (
    "nako-addon-protocol",
    "nako-addon-client",
    "nako-official-addon-catalog",
    "nako",
)
NONSOURCE_REGISTRY_DEPENDENCY_MARKERS = (
    "no matching package named `nako-",
    "failed to select a version for the requirement `nako-",
)


@dataclass(slots=True)
class CommandFailed(RuntimeError):
    command: list[str]
    exit_code: int
    output: str

    def __str__(self) -> str:
        return f"command failed with exit code {self.exit_code}: {' '.join(self.command)}"


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run(command: list[str], cwd: Path, *, check: bool = True) -> subprocess.CompletedProcess[str]:
    print()
    print("==> " + " ".join(command), flush=True)
    result = subprocess.run(
        command,
        cwd=cwd,
        check=False,
        stderr=subprocess.STDOUT,
        stdout=subprocess.PIPE,
        text=True,
    )
    if result.stdout:
        print(result.stdout, end="", flush=True)
    if check and result.returncode != 0:
        raise CommandFailed(command, result.returncode, result.stdout)
    return result


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check or publish Nako's public crates in dependency order."
    )
    parser.add_argument(
        "--mode",
        choices=("dry-run", "publish"),
        default="dry-run",
        help=(
            "Use dry-run for CI checks, publish for crates.io release. Dry-run "
            "uses cargo publish --dry-run for crates without unpublished Nako "
            "dependencies and cargo check for later crates in the first release chain."
        ),
    )
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="Allow local dry-run/package checks with uncommitted changes.",
    )
    parser.add_argument(
        "--registry-settle-seconds",
        type=int,
        default=30,
        help="Seconds to wait after each published crate before publishing the next dependent crate.",
    )
    parser.add_argument(
        "--no-skip-published",
        action="store_true",
        help="In publish mode, fail instead of skipping crates whose current version already exists on crates.io.",
    )
    return parser.parse_args(argv)


def crate_version(root: Path, crate: str) -> str:
    crate_manifest = root / "crates" / crate / "Cargo.toml"
    crate_toml = tomllib.loads(crate_manifest.read_text(encoding="utf-8"))
    version = crate_toml["package"].get("version")
    if isinstance(version, str):
        return version
    if isinstance(version, dict) and version.get("workspace") is True:
        root_toml = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
        return root_toml["workspace"]["package"]["version"]
    raise RuntimeError(f"could not resolve package version for {crate}")


def current_version_is_published(root: Path, crate: str, version: str) -> bool:
    result = run(["cargo", "search", crate, "--limit", "1"], root, check=False)
    if result.returncode != 0:
        print(
            f"warning: could not query crates.io for {crate}; attempting publish",
            flush=True,
        )
        return False

    first_line = next(
        (line.strip() for line in result.stdout.splitlines() if line.strip()),
        "",
    )
    return first_line.startswith(f'{crate} = "{version}"')


def output_is_registry_dependency_gap(output: str) -> bool:
    return any(marker in output for marker in NONSOURCE_REGISTRY_DEPENDENCY_MARKERS)


def dry_run_or_local_check(root: Path, crate: str, dirty_args: list[str]) -> None:
    try:
        run(["cargo", "publish", "-p", crate, "--locked", "--dry-run", *dirty_args], root)
    except CommandFailed as error:
        if not output_is_registry_dependency_gap(error.output):
            raise
        print(
            "    local check: dependent Nako crates are not published to crates.io yet",
            flush=True,
        )
        run(["cargo", "check", "-p", crate, "--all-features", "--tests"], root)


def main(argv: list[str]) -> int:
    args = parse_args(argv)

    root = repo_root()
    print("Nako crates.io publish", flush=True)
    print(f"Mode: {args.mode}", flush=True)

    for crate in PUBLIC_CRATES:
        is_last_crate = crate == PUBLIC_CRATES[-1]
        dirty_args = ["--allow-dirty"] if args.allow_dirty else []
        if args.mode == "dry-run":
            dry_run_or_local_check(root, crate, dirty_args)
        else:
            version = crate_version(root, crate)
            if not args.no_skip_published and current_version_is_published(root, crate, version):
                print(
                    f"Skipping {crate} {version}; current version already exists on crates.io.",
                    flush=True,
                )
                continue
            run(["cargo", "publish", "-p", crate, "--locked", *dirty_args], root)
            if not is_last_crate and args.registry_settle_seconds > 0:
                print(
                    f"Waiting {args.registry_settle_seconds}s for crates.io index propagation...",
                    flush=True,
                )
                time.sleep(args.registry_settle_seconds)

    print(flush=True)
    print("Nako crates.io publish completed.", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

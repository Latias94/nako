#!/usr/bin/env python3
"""Publish Nako's public crates in dependency order."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path


PUBLIC_CRATES = (
    "nako-addon-protocol",
    "nako-addon-client",
    "nako",
)
REGISTRY_INDEPENDENT_DRY_RUN_CRATES = frozenset({"nako-addon-protocol"})


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def run(command: list[str], cwd: Path) -> None:
    print()
    print("==> " + " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, check=True, stderr=subprocess.STDOUT)


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
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)

    if args.mode == "publish" and not os.environ.get("CARGO_REGISTRY_TOKEN"):
        print("CARGO_REGISTRY_TOKEN must be set for crates.io publish.", file=sys.stderr)
        return 64

    root = repo_root()
    print("Nako crates.io publish", flush=True)
    print(f"Mode: {args.mode}", flush=True)

    for crate in PUBLIC_CRATES:
        is_last_crate = crate == PUBLIC_CRATES[-1]
        dirty_args = ["--allow-dirty"] if args.allow_dirty else []
        if args.mode == "dry-run":
            if crate in REGISTRY_INDEPENDENT_DRY_RUN_CRATES:
                run(
                    ["cargo", "publish", "-p", crate, "--locked", "--dry-run", *dirty_args],
                    root,
                )
            else:
                print(
                    "    local check: dependent Nako crates may not exist on crates.io yet",
                    flush=True,
                )
                run(["cargo", "check", "-p", crate, "--all-features", "--tests"], root)
        else:
            run(["cargo", "publish", "-p", crate, "--locked"], root)
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

#!/usr/bin/env python3
"""
Halo2 addition circuit demo (Python driver).

Mirrors halo/src/bin/halo_add_demo.rs: trusted setup -> keygen -> prove -> verify.
Artifacts land in build/halo-add/ by default.

Halo2 proving lives in Rust (halo2_proofs has no stable Python bindings). This script
lets you change witness values in Python without recompiling; it writes prover_input.json
and invokes the prebuilt halo_add_demo binary.

Prerequisites (one-time):
  cd halo && cargo build --release --bin halo_add_demo

Usage:
  python3 scripts/halo_add_plonk_demo.py
  python3 scripts/halo_add_plonk_demo.py --a 10 --b 32
  python3 scripts/halo_add_plonk_demo.py --invalid
  python3 scripts/halo_add_plonk_demo.py --build   # build binary if missing
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
HALO_DIR = REPO_ROOT / "halo"
DEFAULT_OUT_DIR = REPO_ROOT / "build" / "halo-add"
DEFAULT_BIN = HALO_DIR / "target" / "release" / "halo_add_demo"


def log(step: str, title: str) -> None:
    print(f"\n{'=' * 72}", flush=True)
    print(f"STEP {step}: {title}", flush=True)
    print(f"{'=' * 72}", flush=True)


def explain(text: str) -> None:
    print(flush=True)
    for line in text.strip().splitlines():
        print(f"  {line}", flush=True)


def write_json(path: Path, data: object) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def file_report(path: Path) -> None:
    if path.exists():
        rel = path.relative_to(REPO_ROOT) if path.is_relative_to(REPO_ROOT) else path
        print(f"  wrote {rel} ({path.stat().st_size} bytes)", flush=True)


def resolve_binary(explicit: Path | None) -> Path:
    if explicit is not None:
        return explicit
    env_bin = os.environ.get("HALO_ADD_DEMO_BIN")
    if env_bin:
        return Path(env_bin)
    return DEFAULT_BIN


def ensure_binary(bin_path: Path, *, build: bool) -> None:
    if bin_path.is_file():
        return
    if not build:
        raise SystemExit(
            "Missing halo_add_demo binary.\n"
            f"  expected: {bin_path}\n"
            "  one-time build: cd halo && cargo build --release --bin halo_add_demo\n"
            "  or rerun with --build"
        )
    print(f"Building {bin_path.name} (one-time)...")
    subprocess.run(
        ["cargo", "build", "--release", "--bin", "halo_add_demo"],
        cwd=HALO_DIR,
        check=True,
    )
    if not bin_path.is_file():
        raise SystemExit(f"Build finished but binary not found: {bin_path}")


def run_crypto_pipeline(bin_path: Path, out_dir: Path) -> int:
    cmd = [str(bin_path), str(out_dir)]
    print(f"\n$ {' '.join(cmd)}", flush=True)
    return subprocess.run(cmd, cwd=REPO_ROOT, check=False).returncode


def main() -> None:
    parser = argparse.ArgumentParser(description="Halo2 addition PLONK demo (Python)")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=DEFAULT_OUT_DIR,
        help=f"artifact directory (default: {DEFAULT_OUT_DIR.relative_to(REPO_ROOT)})",
    )
    parser.add_argument("--a", type=int, default=3, help="private witness a")
    parser.add_argument("--b", type=int, default=4, help="private witness b")
    parser.add_argument(
        "--invalid",
        action="store_true",
        help="mismatch public c (a+b+1) to demonstrate verification failure",
    )
    parser.add_argument(
        "--bin",
        type=Path,
        default=None,
        help="path to halo_add_demo binary (default: halo/target/release/halo_add_demo)",
    )
    parser.add_argument(
        "--build",
        action="store_true",
        help="cargo build --release if binary is missing",
    )
    args = parser.parse_args()

    out_dir: Path = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    a, b = args.a, args.b
    witness_c = a + b
    public_c = witness_c + 1 if args.invalid else witness_c

    print(f"Output directory: {out_dir}", flush=True)

    log("0", "Configure witness (Python — edit here, no Rust recompile)")
    prover_input = {"a": a, "b": b, "witness_c": witness_c, "public_c": public_c}
    prover_input_path = out_dir / "prover_input.json"
    write_json(prover_input_path, prover_input)
    print(f"  prover_input.json = {json.dumps(prover_input)}", flush=True)
    explain(
        """
        Change --a / --b (or edit prover_input.json) to try new witnesses.
        witness_c is the private advice cell for the sum; public_c is the instance
        column the verifier checks. With --invalid, public_c != a+b so verification fails.
        """
    )

    log("1-4", "Halo2 crypto pipeline (Rust binary)")
    explain(
        """
        Steps inside halo_add_demo:
          1. Params::new(K) -> params.bin (universal SRS)
          2. keygen_vk / keygen_pk from AddCircuit in halo/src/add.rs
          3. create_proof -> proof.bin (prover secrets stay private)
          4. verify_proof as external verifier -> verify_report.json

        Circuit logic is NOT a .circom file; the add gate lives in add.rs configure():
          s * (a + b - c) = 0
        """
    )

    bin_path = resolve_binary(args.bin)
    ensure_binary(bin_path, build=args.build)
    exit_code = run_crypto_pipeline(bin_path, out_dir)

    print("\n[Artifacts]", flush=True)
    for name in (
        "params.bin",
        "manifest.json",
        "prover_input.json",
        "prover_secrets.json",
        "public_inputs.json",
        "proof.bin",
        "verify_report.json",
    ):
        file_report(out_dir / name)

    report_path = out_dir / "verify_report.json"
    if report_path.exists():
        report = json.loads(report_path.read_text(encoding="utf-8"))
        verified = report.get("verified")
        print(flush=True)
        if verified:
            print(f"SUCCESS: {report.get('message', 'verified')}", flush=True)
        else:
            print(f"FAILED: {report.get('message', 'verification rejected')}", flush=True)
            sys.exit(1)

    if exit_code != 0 and not args.invalid:
        sys.exit(exit_code)


if __name__ == "__main__":
    main()

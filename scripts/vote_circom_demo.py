#!/usr/bin/env python3
"""
End-to-end Circom + Groth16 demo for circuits/vote.circom.

Prerequisites (on PATH or via npx):
  - circom (v2.1.x)
  - node
  - npm install   # snarkjs + circomlibjs

Steps:
  1. Compile vote.circom -> r1cs, wasm, sym  (-l circuits for circomlib/)
  2. Generate input.json (Poseidon Merkle path via gen_vote_input.cjs)
  3. Generate witness.wtns
  4. snarkjs wtns check
  5. Powers of Tau (power 14, sized for ~5k constraints)
  6. Groth16 setup, prove, snarkjs verify

Usage:
  python3 scripts/vote_circom_demo.py
  python3 scripts/vote_circom_demo.py --invalid
  python3 scripts/vote_circom_demo.py --witness-only
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
BUILD_DIR = REPO_ROOT / "build" / "vote"
CIRCUIT = REPO_ROOT / "circuits" / "vote.circom"
CIRCOM_INCLUDE = REPO_ROOT / "circuits"
GEN_INPUT = REPO_ROOT / "scripts" / "gen_vote_input.cjs"
POT_POWER = "14"


def log(step: str, message: str) -> None:
    print(f"\n=== [{step}] {message} ===")


def run(
    cmd: list[str], *, input_text: str | None = None, capture: bool = False
) -> subprocess.CompletedProcess:
    display = " ".join(cmd)
    print(f"$ {display}")
    return subprocess.run(
        cmd,
        cwd=REPO_ROOT,
        check=True,
        text=True,
        input=input_text,
        capture_output=capture,
    )


def snarkjs(
    *args: str, input_text: str | None = None, capture: bool = False
) -> subprocess.CompletedProcess:
    return run(["npx", "snarkjs", *args], input_text=input_text, capture=capture)


def write_json(path: Path, data: object) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {path.relative_to(REPO_ROOT)}")


def read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def require_tools() -> None:
    for tool in ("circom", "node"):
        if shutil.which(tool) is None:
            raise SystemExit(f"Missing required tool: {tool}")
    if not (REPO_ROOT / "node_modules" / "snarkjs").is_dir():
        raise SystemExit("Run `npm install` in the repo root first (snarkjs).")
    if not (REPO_ROOT / "node_modules" / "circomlibjs").is_dir():
        raise SystemExit("Run `npm install` in the repo root (needs circomlibjs).")


def compile_circuit() -> None:
    log("1", "Compile vote.circom -> r1cs, wasm, sym")
    BUILD_DIR.mkdir(parents=True, exist_ok=True)
    run([
        "circom",
        str(CIRCUIT),
        "--r1cs",
        "--wasm",
        "--sym",
        "-l",
        str(CIRCOM_INCLUDE),
        "-o",
        str(BUILD_DIR),
    ])
    for name in ("vote.r1cs", "vote_js/vote.wasm", "vote.sym"):
        path = BUILD_DIR / name
        if not path.exists():
            raise SystemExit(f"Expected artifact missing: {path}")


def write_input(invalid: bool) -> Path:
    log("2", "Circuit inputs (Poseidon Merkle + nullifier)")
    input_path = BUILD_DIR / "input.json"
    cmd = ["node", str(GEN_INPUT), str(input_path)]
    if invalid:
        log("2", "Invalid case: wrong Merkle root (witness should fail)")
        cmd.append("--invalid")
    run(cmd)
    payload = read_json(input_path)
    public = {
        "root": payload["root"],
        "nullifierHash": payload["nullifierHash"],
        "vote": payload["vote"],
    }
    print(f"public inputs: {public}")
    print("private: identitySecret + pathElements/pathIndices (depth 20)")
    return input_path


def generate_witness(input_path: Path) -> Path:
    log("3", "Generate witness (wasm program)")
    wasm = BUILD_DIR / "vote_js" / "vote.wasm"
    witness = BUILD_DIR / "witness.wtns"
    run([
        "node",
        str(BUILD_DIR / "vote_js" / "generate_witness.js"),
        str(wasm),
        str(input_path),
        str(witness),
    ])
    return witness


def check_witness(witness: Path) -> None:
    log("4", "Check witness satisfies R1CS")
    snarkjs("wtns", "check", str(BUILD_DIR / "vote.r1cs"), str(witness))


def print_r1cs_info() -> None:
    log("4b", "R1CS statistics")
    snarkjs("r1cs", "info", str(BUILD_DIR / "vote.r1cs"))


def ensure_powers_of_tau() -> Path:
    ptau_final = BUILD_DIR / f"pot{POT_POWER}_final.ptau"
    if ptau_final.exists():
        log("5", f"Reuse existing Powers of Tau: {ptau_final.name}")
        return ptau_final

    log("5", f"Generate local Powers of Tau (bn128, power {POT_POWER})")
    ptau_0000 = BUILD_DIR / f"pot{POT_POWER}_0000.ptau"
    ptau_0001 = BUILD_DIR / f"pot{POT_POWER}_0001.ptau"
    snarkjs("powersoftau", "new", "bn128", POT_POWER, str(ptau_0000))
    snarkjs(
        "powersoftau",
        "contribute",
        str(ptau_0000),
        str(ptau_0001),
        "--name=vote-demo",
        input_text="vote demo entropy line 1\n",
    )
    snarkjs("powersoftau", "prepare", "phase2", str(ptau_0001), str(ptau_final))
    return ptau_final


def groth16_setup(ptau: Path) -> Path:
    log("6", "Groth16 setup (r1cs + ptau -> zkey)")
    zkey_0 = BUILD_DIR / "vote_0000.zkey"
    zkey_final = BUILD_DIR / "vote_final.zkey"
    snarkjs("groth16", "setup", str(BUILD_DIR / "vote.r1cs"), str(ptau), str(zkey_0))
    snarkjs(
        "zkey",
        "contribute",
        str(zkey_0),
        str(zkey_final),
        "--name=vote-demo",
        input_text="vote demo zkey entropy\n",
    )
    return zkey_final


def export_verification_key(zkey_final: Path) -> Path:
    log("7", "Export verification key")
    vk_path = BUILD_DIR / "verification_key.json"
    snarkjs("zkey", "export", "verificationkey", str(zkey_final), str(vk_path))
    return vk_path


def prove_and_verify_snarkjs(
    zkey_final: Path, witness: Path, vk_path: Path
) -> tuple[Path, Path]:
    log("8", "Groth16 prove")
    proof_path = BUILD_DIR / "proof.json"
    public_path = BUILD_DIR / "public.json"
    snarkjs(
        "groth16",
        "prove",
        str(zkey_final),
        str(witness),
        str(proof_path),
        str(public_path),
    )

    public_signals = read_json(public_path)
    print(f"public.json (verifier-visible): {public_signals}")

    log("9", "Groth16 verify (snarkjs + verification_key.json)")
    result = snarkjs(
        "groth16",
        "verify",
        str(vk_path),
        str(public_path),
        str(proof_path),
        capture=True,
    )
    stdout = (result.stdout or "").strip()
    stderr = (result.stderr or "").strip()
    if stdout:
        print(stdout)
    if stderr:
        print(stderr, file=sys.stderr)
    if "OK" not in stdout and "OK" not in stderr:
        raise SystemExit("snarkjs groth16 verify did not report OK")
    print("snarkjs verification: OK")
    return proof_path, public_path


def main() -> None:
    parser = argparse.ArgumentParser(description="Circom vote circuit end-to-end demo")
    parser.add_argument(
        "--invalid",
        action="store_true",
        help="Use a wrong Merkle root so witness generation fails",
    )
    parser.add_argument(
        "--witness-only",
        action="store_true",
        help="Stop after wtns check (no Groth16 setup/prove)",
    )
    args = parser.parse_args()

    require_tools()

    compile_circuit()
    input_path = write_input(args.invalid)

    if args.invalid:
        try:
            generate_witness(input_path)
        except subprocess.CalledProcessError:
            print(
                "\nExpected failure: invalid Merkle root breaks root === merkle.root."
            )
            return
        raise SystemExit("Invalid input should have failed during witness generation.")

    witness = generate_witness(input_path)
    check_witness(witness)
    print_r1cs_info()

    if args.witness_only:
        log("done", f"Witness OK. Artifacts in {BUILD_DIR.relative_to(REPO_ROOT)}/")
        return

    ptau = ensure_powers_of_tau()
    zkey_final = groth16_setup(ptau)
    vk_path = export_verification_key(zkey_final)
    prove_and_verify_snarkjs(zkey_final, witness, vk_path)
    log(
        "done",
        "Pipeline complete: circom -> witness -> wtns check -> groth16 prove -> snarkjs verify",
    )


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
End-to-end Circom + Groth16 demo for circuits/addition.circom.

Prerequisites (on PATH or via npx):
  - circom (v2.1.x)
  - node
  - npm install   # provides snarkjs in node_modules

Steps:
  1. Compile .circom -> .r1cs, .wasm, .sym
  2. Write input.json and generate witness.wtns
  3. Check witness against R1CS
  4. Powers of Tau (local, power 8) if missing
  5. Groth16 setup + zkey contribution
  6. Export verification_key.json and Solidity verifier
  7. Prove and verify with snarkjs
  8. Copy verifier into contracts/, compile with Hardhat, verify on local EVM

Usage:
  python3 scripts/addition_circom_demo.py
  python3 scripts/addition_circom_demo.py --invalid   # demo rejected witness input
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
BUILD_DIR = REPO_ROOT / "build" / "addition"
CIRCUIT = REPO_ROOT / "circuits" / "addition.circom"
CONTRACTS_VERIFIER = REPO_ROOT / "contracts" / "AdditionVerifier.sol"


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
    print(f"Wrote {path.relative_to(REPO_ROOT)}: {data}")


def read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def require_tools() -> None:
    for tool in ("circom", "node"):
        if shutil.which(tool) is None:
            raise SystemExit(f"Missing required tool: {tool}")
    if not (REPO_ROOT / "node_modules" / "snarkjs").is_dir():
        raise SystemExit("Run `npm install` in the repo root first (snarkjs).")


def compile_circuit() -> None:
    log("1", "Compile Circom -> r1cs, wasm, sym")
    BUILD_DIR.mkdir(parents=True, exist_ok=True)
    run([
        "circom",
        str(CIRCUIT),
        "--r1cs",
        "--wasm",
        "--sym",
        "-o",
        str(BUILD_DIR),
    ])
    for name in ("addition.r1cs", "addition_js/addition.wasm", "addition.sym"):
        path = BUILD_DIR / name
        if not path.exists():
            raise SystemExit(f"Expected artifact missing: {path}")


def write_input(invalid: bool) -> Path:
    log("2", "Circuit inputs (public + private)")
    if invalid:
        payload = {"a": "3", "b": "4", "c": "8"}
        log("2", "Invalid case: 3 + 4 != 8 (witness generation should fail)")
    else:
        payload = {"a": "3", "b": "4", "c": "7"}
    input_path = BUILD_DIR / "input.json"
    write_json(input_path, payload)
    return input_path


def generate_witness(input_path: Path) -> Path:
    log("3", "Generate witness (wasm program)")
    wasm = BUILD_DIR / "addition_js" / "addition.wasm"
    witness = BUILD_DIR / "witness.wtns"
    run([
        "node",
        str(BUILD_DIR / "addition_js" / "generate_witness.js"),
        str(wasm),
        str(input_path),
        str(witness),
    ])
    return witness


def check_witness(witness: Path) -> None:
    log("4", "Check witness satisfies R1CS")
    snarkjs("wtns", "check", str(BUILD_DIR / "addition.r1cs"), str(witness))


def print_r1cs_info() -> None:
    log("4b", "R1CS statistics")
    snarkjs("r1cs", "info", str(BUILD_DIR / "addition.r1cs"))


def ensure_powers_of_tau() -> Path:
    ptau_final = BUILD_DIR / "pot8_final.ptau"
    if ptau_final.exists():
        log("5", f"Reuse existing Powers of Tau: {ptau_final.name}")
        return ptau_final

    log("5", "Generate local Powers of Tau (bn128, power 8)")
    ptau_0000 = BUILD_DIR / "pot8_0000.ptau"
    ptau_0001 = BUILD_DIR / "pot8_0001.ptau"
    snarkjs("powersoftau", "new", "bn128", "8", str(ptau_0000))
    snarkjs(
        "powersoftau",
        "contribute",
        str(ptau_0000),
        str(ptau_0001),
        "--name=python-demo",
        input_text="python demo entropy line 1\n",
    )
    snarkjs("powersoftau", "prepare", "phase2", str(ptau_0001), str(ptau_final))
    return ptau_final


def groth16_setup(ptau: Path) -> Path:
    log("6", "Groth16 setup (r1cs + ptau -> zkey)")
    zkey_0 = BUILD_DIR / "addition_0000.zkey"
    zkey_final = BUILD_DIR / "addition_final.zkey"
    snarkjs(
        "groth16", "setup", str(BUILD_DIR / "addition.r1cs"), str(ptau), str(zkey_0)
    )
    snarkjs(
        "zkey",
        "contribute",
        str(zkey_0),
        str(zkey_final),
        "--name=python-demo",
        input_text="python demo zkey entropy\n",
    )
    return zkey_final


def export_keys(zkey_final: Path) -> tuple[Path, Path]:
    log("7", "Export verification key and Solidity verifier")
    vk_path = BUILD_DIR / "verification_key.json"
    generated_verifier = BUILD_DIR / "AdditionVerifier.generated.sol"
    snarkjs("zkey", "export", "verificationkey", str(zkey_final), str(vk_path))
    snarkjs(
        "zkey", "export", "solidityverifier", str(zkey_final), str(generated_verifier)
    )
    shutil.copy2(generated_verifier, CONTRACTS_VERIFIER)
    print(f"Copied verifier -> {CONTRACTS_VERIFIER.relative_to(REPO_ROOT)}")
    return vk_path, generated_verifier


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

    proof = read_json(proof_path)
    public_signals = read_json(public_path)
    print(f"public.json (verifier-visible): {public_signals}")
    print(
        f"proof.json keys: {list(proof.keys()) if isinstance(proof, dict) else proof}"
    )

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


def sync_test_fixtures() -> None:
    """Keep test/fixtures aligned with the zkey/proof from this run."""
    fixtures = REPO_ROOT / "test" / "fixtures"
    for name in ("proof.json", "public.json"):
        src = BUILD_DIR / name
        dst = fixtures / f"addition-{name}"
        shutil.copy2(src, dst)
    print(f"Updated {fixtures.relative_to(REPO_ROOT)}/addition-*.json")


def compile_and_verify_onchain() -> None:
    log("10", "Compile Solidity verifier (Hardhat)")
    run(["npm", "run", "compile"])

    log("11", "Verify proof on local Hardhat EVM")
    run(["npx", "tsx", "scripts/verify-fresh-addition.ts"])
    sync_test_fixtures()


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Circom addition circuit end-to-end demo"
    )
    parser.add_argument(
        "--invalid",
        action="store_true",
        help="Use c=8 so witness generation fails (demo only)",
    )
    parser.add_argument(
        "--skip-onchain",
        action="store_true",
        help="Stop after snarkjs groth16 verify (no Hardhat compile)",
    )
    args = parser.parse_args()

    require_tools()

    compile_circuit()
    input_path = write_input(args.invalid)

    if args.invalid:
        try:
            generate_witness(input_path)
        except subprocess.CalledProcessError:
            print("\nExpected failure: invalid input cannot satisfy sum === c.")
            return
        raise SystemExit("Invalid input should have failed during witness generation.")

    witness = generate_witness(input_path)
    check_witness(witness)
    print_r1cs_info()

    ptau = ensure_powers_of_tau()
    zkey_final = groth16_setup(ptau)
    export_keys(zkey_final)
    prove_and_verify_snarkjs(zkey_final, witness, BUILD_DIR / "verification_key.json")

    if args.skip_onchain:
        log("done", f"Artifacts in {BUILD_DIR.relative_to(REPO_ROOT)}/")
        return

    compile_and_verify_onchain()
    log(
        "done",
        "Pipeline complete: circom -> witness -> groth16 -> snarkjs verify -> Solidity compile -> on-chain verify",
    )


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""
End-to-end Circom + PLONK demo for circuits/addition.circom.

Same circuit and witness path as addition_circom_demo.py, but uses snarkjs PLONK
instead of Groth16. Each step prints its output and a short explanation.

Prerequisites:
  - circom, node, npm install (snarkjs)

Usage:
  python3 scripts/addition_plonk_demo.py
  python3 scripts/addition_plonk_demo.py --invalid
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
BUILD_DIR = REPO_ROOT / "build" / "addition-plonk"
CIRCUIT = REPO_ROOT / "circuits" / "addition.circom"
POT_POWER = "8"


def log(step: str, title: str) -> None:
    print(f"\n{'=' * 72}")
    print(f"STEP {step}: {title}")
    print(f"{'=' * 72}")


def explain(text: str) -> None:
    print("\n[Explanation]")
    for line in text.strip().splitlines():
        print(f"  {line}")


def run(
    cmd: list[str], *, input_text: str | None = None, capture: bool = False
) -> subprocess.CompletedProcess:
    print(f"\n$ {' '.join(cmd)}")
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


def read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def file_report(path: Path) -> None:
    if path.exists():
        print(
            f"  artifact: {path.relative_to(REPO_ROOT)} ({path.stat().st_size} bytes)"
        )


def require_tools() -> None:
    for tool in ("circom", "node"):
        if shutil.which(tool) is None:
            raise SystemExit(f"Missing required tool: {tool}")
    if not (REPO_ROOT / "node_modules" / "snarkjs").is_dir():
        raise SystemExit("Run `npm install` in the repo root first (snarkjs).")


def step1_compile() -> None:
    log("1", "Compile addition.circom")
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
    print("\n[Result]")
    for name in ("addition.r1cs", "addition_js/addition.wasm", "addition.sym"):
        file_report(BUILD_DIR / name)
    explain(
        """
        Circom turns the source circuit into:
        - addition.r1cs: constraint system specification for the prover backend
        - addition.wasm: executable witness calculator
        - addition.sym: human-readable signal names for debugging
        PLONK and Groth16 share the same Circom compile output up to this point.
        """
    )


def step2_input(invalid: bool) -> Path:
    log("2", "Write circuit input (public + private)")
    payload = (
        {"a": "3", "b": "4", "c": "8"} if invalid else {"a": "3", "b": "4", "c": "7"}
    )
    input_path = BUILD_DIR / "input.json"
    write_json(input_path, payload)
    print("\n[Result]")
    print(f"  input.json = {json.dumps(payload)}")
    if invalid:
        print("  mode: INVALID (3 + 4 != 8)")
    explain(
        """
        The prover claims: "I know private a, b such that a + b = public c."
        - private witness inputs: a, b
        - public input: c (listed in component main { public [c] })
        The wasm program will reject assignments that violate sum === c.
        """
    )
    return input_path


def step3_witness(input_path: Path) -> Path:
    log("3", "Generate witness with addition.wasm")
    wasm = BUILD_DIR / "addition_js" / "addition.wasm"
    witness = BUILD_DIR / "witness.wtns"
    run([
        "node",
        str(BUILD_DIR / "addition_js" / "generate_witness.js"),
        str(wasm),
        str(input_path),
        str(witness),
    ])
    print("\n[Result]")
    file_report(witness)
    explain(
        """
        witness.wtns stores the full assignment of circuit wires for this input.
        It is deterministic: same input.json always yields the same witness.
        Prover blinding randomness is NOT stored here; it is chosen later
        during the proof step.
        """
    )
    return witness


def step4_check_witness(witness: Path) -> None:
    log("4", "Check witness against R1CS")
    result = snarkjs(
        "wtns",
        "check",
        str(BUILD_DIR / "addition.r1cs"),
        str(witness),
        capture=True,
    )
    output = (result.stdout or "") + (result.stderr or "")
    print("\n[Result]")
    for line in output.splitlines():
        if line.strip():
            print(f"  {line}")
    explain(
        """
        snarkjs wtns check confirms the witness satisfies the R1CS constraints.
        This is pre-proof sanity checking: "does this assignment obey the circuit?"
        For the addition demo, invalid input usually fails earlier in wasm;
        this step validates the witness file format and constraint satisfaction.
        """
    )


def step5_r1cs_info() -> None:
    log("5", "Inspect R1CS size")
    result = snarkjs("r1cs", "info", str(BUILD_DIR / "addition.r1cs"), capture=True)
    output = (result.stdout or "") + (result.stderr or "")
    print("\n[Result]")
    for line in output.splitlines():
        if line.strip():
            print(f"  {line}")
    explain(
        """
        R1CS statistics tell the prover how large the setup must be.
        snarkjs reports "Plonk constraints" separately during PLONK setup.
        Linear-only pieces of a Circom circuit may not appear as multiplication
        gates in the R1CS file, but PLONK still builds a proof system from it.
        """
    )


def step6_powers_of_tau() -> Path:
    log("6", "Powers of Tau (universal SRS)")
    ptau_final = BUILD_DIR / f"pot{POT_POWER}_final.ptau"
    if ptau_final.exists():
        print("\n[Result]")
        file_report(ptau_final)
        explain(
            """
            Reusing an existing ptau file. Powers of Tau is a universal ceremony
            parameter: [tau^0, tau^1, ...] in the curve groups. PLONK can reuse
            the same ptau across many different circuits (universal setup), unlike
            the circuit-specific Groth16 phase-2 in many workflows.
            """
        )
        return ptau_final

    ptau_0000 = BUILD_DIR / f"pot{POT_POWER}_0000.ptau"
    ptau_0001 = BUILD_DIR / f"pot{POT_POWER}_0001.ptau"
    snarkjs("powersoftau", "new", "bn128", POT_POWER, str(ptau_0000))
    snarkjs(
        "powersoftau",
        "contribute",
        str(ptau_0000),
        str(ptau_0001),
        "--name=plonk-demo",
        input_text="plonk demo entropy\n",
    )
    snarkjs("powersoftau", "prepare", "phase2", str(ptau_0001), str(ptau_final))
    print("\n[Result]")
    file_report(ptau_final)
    explain(
        """
        Generated a local Powers of Tau file (bn128, power 8) for this demo.
        Production systems often download a public final ptau instead.
        PLONK setup below consumes this file to build a circuit-specific proving key.
        """
    )
    return ptau_final


def step7_plonk_setup(ptau: Path) -> Path:
    log("7", "PLONK setup (r1cs + ptau -> zkey)")
    zkey = BUILD_DIR / "addition_plonk.zkey"
    result = snarkjs(
        "plonk",
        "setup",
        str(BUILD_DIR / "addition.r1cs"),
        str(ptau),
        str(zkey),
        capture=True,
    )
    output = (result.stdout or "") + (result.stderr or "")
    print("\n[Result]")
    for line in output.splitlines():
        if line.strip():
            print(f"  {line}")
    file_report(zkey)
    explain(
        """
        PLONK setup creates addition_plonk.zkey (proving / verification key material).
        In snarkjs this is one command; there is no separate Groth16-style
        zkey contribute step for this demo path.
        The log line "Plonk constraints" is the size of the PLONK gate system
        snarkjs builds from the R1CS.
        """
    )
    return zkey


def step8_export_vk(zkey: Path) -> Path:
    log("8", "Export verification key")
    vk_path = BUILD_DIR / "verification_key.json"
    result = snarkjs(
        "zkey",
        "export",
        "verificationkey",
        str(zkey),
        str(vk_path),
        capture=True,
    )
    output = (result.stdout or "") + (result.stderr or "")
    vk = read_json(vk_path)
    print("\n[Result]")
    for line in output.splitlines():
        if line.strip():
            print(f"  {line}")
    print(f"  protocol: {vk.get('protocol')}")
    print(f"  curve: {vk.get('curve')}")
    print(f"  nPublic: {vk.get('nPublic')}")
    print(f"  vk keys: {', '.join(sorted(vk.keys()))}")
    explain(
        """
        The verification key encodes the PLONK gate polynomials (Ql, Qr, Qo, Qc, Qm),
        permutation polynomials (S1, S2, S3), and public-input metadata.
        The verifier uses this JSON together with public signals and the proof.
        Groth16 vk is much smaller; PLONK vk carries the custom gate definition.
        """
    )
    return vk_path


def step9_prove(zkey: Path, witness: Path) -> tuple[Path, Path]:
    log("9", "PLONK prove")
    proof_path = BUILD_DIR / "proof.json"
    public_path = BUILD_DIR / "public.json"
    snarkjs(
        "plonk",
        "prove",
        str(zkey),
        str(witness),
        str(proof_path),
        str(public_path),
    )
    proof = read_json(proof_path)
    public_signals = read_json(public_path)
    print("\n[Result]")
    print(f"  public.json = {public_signals}")
    print(f"  proof size = {proof_path.stat().st_size} bytes")
    print(f"  proof fields = {', '.join(sorted(proof.keys()))}")
    print("  sample proof points:")
    for key in ("A", "B", "C", "Z"):
        if key in proof:
            print(
                f"    {key} = ({proof[key][0][:16]}..., {proof[key][1][:16]}..., {proof[key][2]})"
            )
    if "eval_a" in proof:
        print(f"    eval_a = {proof['eval_a']}")
        print(f"    eval_b = {proof['eval_b']}")
        print(f"    eval_c = {proof['eval_c']}")
    explain(
        """
        PLONK proof contains KZG commitments (A, B, C, Z, T1, T2, T3, Wxi, Wxiw)
        plus opening evaluations at challenge points.
        public.json holds only the verifier-visible value c = 7.
        Private a, b remain hidden; the proof shows the relation is satisfiable.
        Compared to Groth16's 3 group elements, PLONK proofs are larger but use
        a universal setup and polynomial commitment checks instead of pairings.
        """
    )
    return proof_path, public_path


def step10_verify(vk_path: Path, proof_path: Path, public_path: Path) -> None:
    log("10", "PLONK verify")
    result = snarkjs(
        "plonk",
        "verify",
        str(vk_path),
        str(public_path),
        str(proof_path),
        capture=True,
    )
    output = (result.stdout or "") + (result.stderr or "")
    print("\n[Result]")
    for line in output.splitlines():
        if line.strip():
            print(f"  {line}")
    if "OK" not in output:
        raise SystemExit("snarkjs plonk verify did not report OK")
    explain(
        """
        Verification checks the PLONK polynomial identities using the proof
        commitments and public inputs, without learning the witness.
        Success means: "there exists private a, b such that a + b equals public c."
        """
    )


def print_groth16_comparison() -> None:
    groth16_proof = REPO_ROOT / "build" / "addition" / "proof.json"
    plonk_proof = BUILD_DIR / "proof.json"
    print(f"\n{'=' * 72}")
    print("COMPARISON: same circuit, different proof systems")
    print(f"{'=' * 72}")
    if groth16_proof.exists():
        g = read_json(groth16_proof)
        p = read_json(plonk_proof)
        print(f"  Groth16 proof keys: {sorted(g.keys())}")
        print(f"  PLONK proof keys:   {sorted(p.keys())}")
        print(f"  Groth16 proof size: {groth16_proof.stat().st_size} bytes")
        print(f"  PLONK proof size:   {plonk_proof.stat().st_size} bytes")
    else:
        print(
            "  Run scripts/addition_circom_demo.py to generate Groth16 artifacts for size comparison."
        )
    explain(
        """
        Same addition.circom, same witness idea, different proof backend:
        - Groth16: pairing-based, circuit-specific trusted setup, compact proof
        - PLONK: polynomial / KZG style, universal SRS, larger proof, flexible gates
        Both hide a, b and only expose public c in public.json.
        """
    )


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Circom addition circuit demo with PLONK (snarkjs)"
    )
    parser.add_argument(
        "--invalid",
        action="store_true",
        help="Use c=8 so witness generation should fail",
    )
    args = parser.parse_args()

    require_tools()

    step1_compile()
    input_path = step2_input(args.invalid)

    if args.invalid:
        try:
            step3_witness(input_path)
        except subprocess.CalledProcessError:
            print("\n[Result] Expected failure during witness generation.")
            explain(
                """
                Invalid public sum breaks the constraint sum === c inside wasm.
                No witness file means no PLONK proof can be generated.
                """
            )
            return
        raise SystemExit("Invalid input should have failed during witness generation.")

    witness = step3_witness(input_path)
    step4_check_witness(witness)
    step5_r1cs_info()
    ptau = step6_powers_of_tau()
    zkey = step7_plonk_setup(ptau)
    vk_path = step8_export_vk(zkey)
    step9_prove(zkey, witness)
    step10_verify(vk_path, BUILD_DIR / "proof.json", BUILD_DIR / "public.json")
    print_groth16_comparison()
    log("done", f"Artifacts written to {BUILD_DIR.relative_to(REPO_ROOT)}/")


if __name__ == "__main__":
    main()

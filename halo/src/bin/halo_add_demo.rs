//! External-user flow for the Halo2 addition circuit:
//! setup (SRS) -> prover (proof) -> verifier (verify).
//!
//! Artifacts are written under `--out-dir` (default: `build/halo-add/`).

use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use halo2_proofs::pasta::{EqAffine, Fp};
use halo2_proofs::plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, SingleVerifier};
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::transcript::{Blake2bRead, Blake2bWrite, Challenge255};
use halo_gate::{AddCircuit, ADD_CIRCUIT_K};
use rand_core::OsRng;

const DEFAULT_OUT_DIR: &str = "build/halo-add";

struct ProverInput {
    a: u64,
    b: u64,
    /// Public instance value; defaults to `a + b` when omitted.
    public_c: u64,
    /// Private advice value for `c` in the circuit; defaults to `a + b`.
    witness_c: u64,
}

fn load_prover_input(out_dir: &Path) -> ProverInput {
    let path = out_dir.join("prover_input.json");
    if !path.exists() {
        return ProverInput {
            a: 3,
            b: 4,
            public_c: 7,
            witness_c: 7,
        };
    }

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read prover_input.json")).expect(
            "parse prover_input.json",
        );
    let a = json["a"].as_u64().expect("prover_input.a");
    let b = json["b"].as_u64().expect("prover_input.b");
    let sum = a + b;
    let witness_c = json
        .get("witness_c")
        .and_then(|v| v.as_u64())
        .unwrap_or(sum);
    let public_c = json
        .get("public_c")
        .and_then(|v| v.as_u64())
        .unwrap_or(sum);

    ProverInput {
        a,
        b,
        public_c,
        witness_c,
    }
}

fn default_out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("halo crate lives in repo")
        .join(DEFAULT_OUT_DIR)
}

fn main() {
    let out_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_out_dir);

    if let Err(err) = run_pipeline(&out_dir) {
        eprintln!("halo_add_demo failed: {err}");
        std::process::exit(1);
    }
}

fn run_pipeline(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(out_dir)?;

    let witness = load_prover_input(out_dir);

    println!("Output directory: {}", out_dir.display());
    println!(
        "Witness: a={}, b={}, witness_c={}, public_c={}",
        witness.a, witness.b, witness.witness_c, witness.public_c
    );
    println!();

    let params_path = out_dir.join("params.bin");
    let manifest_path = out_dir.join("manifest.json");
    let prover_secrets_path = out_dir.join("prover_secrets.json");
    let public_inputs_path = out_dir.join("public_inputs.json");
    let proof_path = out_dir.join("proof.bin");
    let verify_report_path = out_dir.join("verify_report.json");

    // -------------------------------------------------------------------------
    // Step 1 — Trusted setup: universal SRS (KZG params)
    // -------------------------------------------------------------------------
    step_header(
        "1",
        "Trusted setup (Params / SRS)",
        "Generate the polynomial commitment parameters. In production this is a \
         ceremony; here we use Params::new locally. Same params.bin is shared with \
         prover and verifier.",
    );

    let params = Params::<EqAffine>::new(ADD_CIRCUIT_K);
    let mut params_file = BufWriter::new(File::create(&params_path)?);
    params.write(&mut params_file)?;
    params_file.flush()?;

    print_file(&params_path);
    println!();

    // -------------------------------------------------------------------------
    // Step 2 — Circuit keys (vk + pk) derived from circuit structure
    // -------------------------------------------------------------------------
    step_header(
        "2",
        "Keygen (vk + pk)",
        "Derive verifying key and proving key from the empty circuit layout. \
         Halo2 0.3 does not ship vk/pk file serde here; both parties re-derive vk \
         from params.bin + the published circuit code. Only the prover needs pk.",
    );

    let empty_circuit = AddCircuit {
        a: Fp::zero(),
        b: Fp::zero(),
        c: Fp::zero(),
    };
    let vk = keygen_vk(&params, &empty_circuit)?;
    let pk = keygen_pk(&params, vk, &empty_circuit)?;

    write_json(
        &manifest_path,
        &serde_json::json!({
            "circuit": "addition a + b = c (public c)",
            "protocol": "halo2_plonk",
            "curve": "pallas/Vesta (pasta EqAffine)",
            "k": ADD_CIRCUIT_K,
            "rows": 1u64 << ADD_CIRCUIT_K,
            "public_inputs": ["c"],
            "artifacts": {
                "params": "params.bin",
                "proof": "proof.bin",
                "public_inputs": "public_inputs.json"
            },
            "note": "vk/pk are regenerated from params.bin + AddCircuit source; not stored as separate files in this demo."
        }),
    )?;
    print_file(&manifest_path);
    println!();

    // -------------------------------------------------------------------------
    // Step 3 — Prover: private witness + public c -> proof
    // -------------------------------------------------------------------------
    step_header(
        "3",
        "Prover",
        &format!(
            "Prover knows private a={}, b={} and assigns witness c={}. Public instance \
             c={}. Creates a PLONK proof with create_proof. Verifier will never receive a or b.",
            witness.a, witness.b, witness.witness_c, witness.public_c
        ),
    );

    let prover_circuit = AddCircuit {
        a: Fp::from(witness.a),
        b: Fp::from(witness.b),
        c: Fp::from(witness.witness_c),
    };
    let public_c = Fp::from(witness.public_c);

    write_json(
        &prover_secrets_path,
        &serde_json::json!({
            "a": witness.a.to_string(),
            "b": witness.b.to_string(),
            "witness_c": witness.witness_c.to_string(),
            "note": "prover-only; do not send to verifier"
        }),
    )?;
    write_json(
        &public_inputs_path,
        &serde_json::json!([witness.public_c.to_string()]),
    )?;
    print_file(&prover_secrets_path);
    print_file(&public_inputs_path);

    let mut transcript = Blake2bWrite::<_, EqAffine, Challenge255<_>>::init(vec![]);
    create_proof(
        &params,
        &pk,
        &[prover_circuit],
        &[&[&[public_c]]],
        OsRng,
        &mut transcript,
    )?;
    let proof = transcript.finalize();
    fs::write(&proof_path, &proof)?;
    print_file(&proof_path);
    println!();

    // -------------------------------------------------------------------------
    // Step 4 — Verifier: params + vk + public inputs + proof
    // -------------------------------------------------------------------------
    step_header(
        "4",
        "Verifier (external party)",
        "Verifier reads params.bin, public_inputs.json, proof.bin, re-derives vk \
         from the same circuit, and runs verify_proof. No access to a or b.",
    );

    let verify_ok = verify_as_external_party(
        &params_path,
        &public_inputs_path,
        &proof_path,
        &empty_circuit,
    )?;

    write_json(
        &verify_report_path,
        &serde_json::json!({
            "verified": verify_ok,
            "public_inputs": [witness.public_c.to_string()],
            "message": if verify_ok {
                format!(
                    "Proof accepted: exists private a,b with a+b={}",
                    witness.public_c
                )
            } else {
                "Proof rejected".to_string()
            }
        }),
    )?;
    print_file(&verify_report_path);

    println!();
    if verify_ok {
        println!("SUCCESS: external verifier accepted the proof.");
    } else {
        return Err("verification failed".into());
    }

    Ok(())
}

fn verify_as_external_party(
    params_path: &Path,
    public_inputs_path: &Path,
    proof_path: &Path,
    empty_circuit: &AddCircuit,
) -> Result<bool, Box<dyn std::error::Error>> {
    let params = {
        let mut reader = BufReader::new(File::open(params_path)?);
        Params::<EqAffine>::read(&mut reader)?
    };

    let public_inputs_json: Vec<String> =
        serde_json::from_str(&fs::read_to_string(public_inputs_path)?)?;
    let public_c = Fp::from(public_inputs_json[0].parse::<u64>()?);

    let vk = keygen_vk(&params, empty_circuit)?;
    let proof = fs::read(proof_path)?;

    let strategy = SingleVerifier::new(&params);
    let mut transcript = Blake2bRead::<_, EqAffine, Challenge255<_>>::init(&proof[..]);
    Ok(verify_proof(&params, &vk, strategy, &[&[&[public_c]]], &mut transcript).is_ok())
}

fn step_header(step: &str, title: &str, explanation: &str) {
    println!("{}", "=".repeat(72));
    println!("STEP {step}: {title}");
    println!("{}", "=".repeat(72));
    println!("{explanation}");
    println!();
}

fn print_file(path: &Path) {
    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    println!("  wrote {} ({} bytes)", path.display(), size);
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}

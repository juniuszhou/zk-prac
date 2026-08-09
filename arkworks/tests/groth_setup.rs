/*
 * Educational demonstration of the Groth16 zk-SNARK setup process using arkworks.
 *
 * This file walks through every step of Groth16 in detail:
 *   1. Elliptic curve groups G1, G2 and the target group GT
 *   2. The pairing function e: G1 × G2 → GT
 *   3. The "toxic waste" — random scalars α, β, γ, δ, τ that must be destroyed
 *   4. Proving key (pk) generation from toxic waste + QAP polynomials
 *   5. Verifying key (vk) generation for public verification
 *   6. Proof generation (prove)
 *   7. Proof verification via the pairing equation
 *
 * Every group element is annotated with its mathematical meaning.
 */

use ark_bls12_381::{Bls12_381, Fr, G1Projective as G1, G2Projective as G2};
use ark_ec::{pairing::Pairing, CurveGroup, Group};
use ark_ff::{PrimeField, UniformRand};
use ark_groth16::{Groth16, PreparedVerifyingKey};
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::SNARK;
use ark_std::Zero;
use rand::rngs::StdRng;
use rand::SeedableRng;

// ============================================================================
// Circuit: a + b = c
//   Public input:  c
//   Private witness: a, b
//
// R1CS encoding (low-level API):
//   Variables: [ONE(0), c(1), a(2), b(3)]
//   Constraint: (a + b) * 1 = c
//     A = a + b,   B = 1,   C = c
//     ⟨A,w⟩·⟨B,w⟩ = ⟨C,w⟩  →  (a+b)·1 = c
// ============================================================================

#[derive(Clone)]
struct AddCircuit {
    a: Option<Fr>,
    b: Option<Fr>,
    c: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for AddCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let a_var = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b_var = cs.new_witness_variable(|| self.b.ok_or(SynthesisError::AssignmentMissing))?;
        let c_var = cs.new_input_variable(|| self.c.ok_or(SynthesisError::AssignmentMissing))?;

        cs.enforce_constraint(
            ark_relations::r1cs::LinearCombination::<Fr>::from(a_var) + b_var,
            ark_relations::r1cs::LinearCombination::<Fr>::from(ark_relations::r1cs::Variable::One),
            ark_relations::r1cs::LinearCombination::<Fr>::from(c_var),
        )?;
        Ok(())
    }
}

// ============================================================================
// Helper: print a G1 affine point with explanation
// ============================================================================
fn print_g1(label: &str, pt: impl Into<G1>) {
    let aff: ark_bls12_381::G1Affine = pt.into().into_affine();
    println!("  {}: {}", label, aff);
    // In G1, coordinates are in Fq (the base field).
    // The point is on the curve y² = x³ + β·(x+2) over Fq (BLS12-381 specific).
}

// ============================================================================
// Helper: print a G2 affine point with explanation
// ============================================================================
fn print_g2(label: &str, pt: impl Into<G2>) {
    let aff: ark_bls12_381::G2Affine = pt.into().into_affine();
    println!("  {}: {}", label, aff);
    // In G2, coordinates live in Fq² (quadratic extension field).
    // Each coordinate is (c0, c1) representing c0 + c1·i where i² = -1 over Fq.
    // G2 points satisfy a twisted curve equation over Fq².
}

// ============================================================================
// Helper: print a scalar value with explanation
// ============================================================================
fn print_fr(label: &str, val: Fr) {
    // Print as big integer so beginners can see the actual modular value
    let bigint = val.into_bigint();
    println!(
        "  {} = {} (mod p)",
        label,
        num_bigint::BigUint::from(bigint)
    );
}

// ============================================================================
// Demonstration entry point
// ============================================================================
pub fn run_groth16_setup_demo() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Groth16 zk-SNARK Setup — Full Educational Demo       ║");
    println!("║       Library: arkworks (ark-bls12-381)                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Use a deterministic seed so output is reproducible
    let mut rng = StdRng::seed_from_u64(12345);

    // ------------------------------------------------------------------
    // PART 0: Elliptic Curve Groups and Pairing
    // ------------------------------------------------------------------
    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PART 0: Elliptic Curve Groups & Pairing                    │");
    println!("└─────────────────────────────────────────────────────────────┘");

    // BLS12-381 has two elliptic curve groups:
    //
    //   G1 — points on y² = x³ + 4 over Fq (base field, 381 bits)
    //        Generator: a fixed known point on the curve
    //        Scalar multiplication: P * s where s ∈ Fr (scalar field, 255 bits)
    //
    //   G2 — points on a "twisted" curve over Fq² (extension field)
    //        Generator: a fixed known point, coordinates in Fq²
    //        Scalar multiplication works the same way: Q * s where s ∈ Fr
    //
    //   GT — the "target group", a multiplicative subgroup of Fq¹²
    //        Pairing: e: G1 × G2 → GT  (bilinear map)
    //
    // The BLS12 suffix means: embedding degree k=12.
    // The 381 means: ~381-bit prime modulus for Fq.

    let g1 = G1::generator();
    let g2 = G2::generator();

    println!("\n  Curve: BLS12-381");
    println!(
        "  Base field Fq bit-size: {}",
        ark_bls12_381::Fq::MODULUS_BIT_SIZE
    );
    println!("  Scalar field Fr bit-size: {}", Fr::MODULUS_BIT_SIZE);
    println!("  Embedding degree: 12 (Fq¹² target group)");

    println!("\n  G1 generator (on curve y² = x³ + 4 over Fq):");
    print_g1("    g1 (α·G₁ when α=1)", g1);

    println!("\n  G2 generator (twisted curve over Fq²):");
    print_g2("    g2 (β·G₂ when β=1)", g2);

    // Show scalar multiplication: g1 * 5
    let g1_times_5 = g1 * Fr::from(5u64);
    println!("\n  Scalar multiplication: g1 × 5 ∈ G1");
    print_g1("    g1·5", g1_times_5);

    // Show scalar multiplication: g2 * 7
    let g2_times_7 = g2 * Fr::from(7u64);
    println!("\n  Scalar multiplication: g2 × 7 ∈ G2");
    print_g2("    g2·7", g2_times_7);

    // Demonstrate the pairing — the core mathematical tool
    //
    // The pairing e: G1 × G2 → GT is a bilinear map with three key properties:
    //
    //   1. BILINEAR:   e(a·P, b·Q) = e(P, Q)^(a·b)
    //                  Equivalently: e(a·P, Q) = e(P, Q)^a = e(P, b·Q)^b
    //                  This is the property that makes all ZK proofs work.
    //
    //   2. NON-DEGENERATE:  e(g1, g2) ≠ 1 (the identity element of GT)
    //                      If the result were always 1, the pairing would be useless.
    //
    //   3. EFFICIENTLY COMPUTABLE:  There exists a fast algorithm (Miller loop +
    //                      final exponentiation) to compute e(P, Q) for any P∈G1, Q∈G2.
    //
    // Why pairings matter for Groth16:
    //   The verifier checks:  e(A, B) = e(α, β) · e(C, δ) · e(public_inputs, ...)
    //   This single equation simultaneously verifies ALL constraints without
    //   revealing any private data. The bilinearity lets the verifier check
    //   polynomial relationships using only group elements.

    let pairing_result = Bls12_381::pairing(g1, g2);
    println!("\n  Pairing: e(g1, g2) ∈ GT (target group)");
    println!("    e(g1, g2) = {:?}", pairing_result);

    // Demonstrate bilinearity: e(5·g1, 7·g2) = e(g1, g2)^(5·7)
    let left = Bls12_381::pairing(g1_times_5, g2_times_7);
    let right = pairing_result * (Fr::from(35u64));
    assert_eq!(left, right, "Bilinearity check failed!");
    println!("\n  Bilinearity check: e(5·g1, 7·g2) == e(g1, g2)^(35) ✓");

    // Non-degeneracy
    assert!(!pairing_result.is_zero(), "Pairing should not be zero!");
    println!("  Non-degeneracy: e(g1, g2) ≠ 1 (identity) ✓");

    // ------------------------------------------------------------------
    // PART 1: The Circuit and R1CS
    // ------------------------------------------------------------------
    println!("\n\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PART 1: Circuit → R1CS (Rank-1 Constraint System)         │");
    println!("└─────────────────────────────────────────────────────────────┘");

    // The circuit: prove knowledge of a, b such that a + b = c
    let a_val = Fr::from(3u64);
    let b_val = Fr::from(5u64);
    let c_val = Fr::from(8u64);

    let circuit = AddCircuit {
        a: Some(a_val),
        b: Some(b_val),
        c: Some(c_val),
    };

    println!("\n  Circuit: a + b = c");
    println!("    Public input (known to verifier):  c = {}", 8u64);
    println!(
        "    Private witness (known only to prover): a = {}, b = {}",
        3u64, 5u64
    );

    // Build the R1CS constraint system manually to show the matrices
    let cs = ark_relations::r1cs::ConstraintSystem::<Fr>::new_ref();
    circuit.clone().generate_constraints(cs.clone()).unwrap();
    cs.finalize();

    let matrices = cs.to_matrices().expect("to_matrices");
    println!("\n  R1CS Matrices (sparse form):");
    println!("    Number of constraints: {}", matrices.num_constraints);
    println!(
        "    Instance variables (public + ONE): {}",
        matrices.num_instance_variables
    );
    println!(
        "    Witness variables (private): {}",
        matrices.num_witness_variables
    );

    // Variable layout:
    //   index 0: ONE  (implicit constant 1)
    //   index 1: c    (public input)
    //   index 2: a    (witness)
    //   index 3: b    (witness)
    //
    // Constraint: (a + b) * 1 = c
    //   Matrix A row 0: [0:0, 2:1, 3:1]  →  0·ONE + 1·a + 1·b = a + b
    //   Matrix B row 0: [0:1]             →  1·ONE = 1
    //   Matrix C row 0: [1:1]             →  1·c = c
    println!("\n  Constraint 0: (a + b) × 1 = c");
    println!("    A-row: a + b    (linear combination of variables)");
    println!("    B-row: 1        (just the implicit constant ONE)");
    println!("    C-row: c        (result)");
    println!(
        "    Check: (a+b)·1 = c  →  ({}+{})·1 = {} ✓",
        3u64, 5u64, 8u64
    );

    // ------------------------------------------------------------------
    // PART 2: R1CS → QAP (Quadratic Arithmetic Program)
    // ------------------------------------------------------------------
    println!("\n\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PART 2: R1CS → QAP (Polynomial Encoding)                  │");
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("\n  The R1CS matrices A, B, C are converted to polynomials");
    println!("  via Lagrange interpolation over an evaluation domain.");
    println!("  For each variable column j, we get polynomials A_j(x), B_j(x), C_j(x).");
    println!("\n  The QAP property:");
    println!("    (Σ A_j(t)·z_j) · (Σ B_j(t)·z_j) − Σ C_j(t)·z_j  ≡  0  (mod Z_H(t))");
    println!("  where z = (1, c, a, b) is the witness vector,");
    println!("  t is a random evaluation point, and Z_H is the vanishing polynomial.");
    println!("\n  If the quotient is divisible by Z_H, then ALL constraints are satisfied.");

    // ------------------------------------------------------------------
    // PART 3: Trusted Setup — Generate Toxic Waste
    // ------------------------------------------------------------------
    println!("\n\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PART 3: Trusted Setup — 'Toxic Waste'                     │");
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("\n  Groth16 setup samples random scalars called 'toxic waste'.");
    println!("  These MUST be destroyed after setup. If anyone knows them,");
    println!("  they can forge fake proofs for ANY statement.");
    println!("\n  The toxic waste values are:");

    // Sample the toxic waste — these are the secret random scalars
    let alpha: Fr = Fr::rand(&mut rng);
    let beta: Fr = Fr::rand(&mut rng);
    let gamma: Fr = Fr::rand(&mut rng);
    let delta: Fr = Fr::rand(&mut rng);

    print_fr("    α (alpha)", alpha);
    print_fr("    β (beta)", beta);
    print_fr("    γ (gamma)", gamma);
    print_fr("    δ (delta)", delta);

    // Also sample τ (tau) — used for the QAP evaluation point
    let domain_size = matrices.num_constraints + matrices.num_instance_variables;
    let domain =
        GeneralEvaluationDomain::<Fr>::new(domain_size).expect("domain construction failed");
    let tau: Fr = domain.sample_element_outside_domain(&mut rng);

    print_fr("    τ (tau, QAP evaluation point)", tau);

    println!("\n  ⚠️  ALL of these values must be securely deleted after setup!");
    println!("  ⚠️  If leaked, an attacker can create proofs of false statements.");

    // ------------------------------------------------------------------
    // PART 4: Proving Key Generation
    // ------------------------------------------------------------------
    println!("\n\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PART 4: Proving Key (pk) Generation                       │");
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("\n  The proving key contains group elements derived from the");
    println!("  toxic waste and the QAP polynomials. Each element is a");
    println!("  scalar multiplication of the curve generators.");

    // Now actually run the real Groth16 setup to get pk and vk
    // We'll use the same circuit but a fresh RNG to get proper keys
    let mut setup_rng = StdRng::seed_from_u64(99999);

    let (pk, vk) =
        <Groth16<Bls12_381> as SNARK<Fr>>::circuit_specific_setup(circuit.clone(), &mut setup_rng)
            .expect("Groth16 setup failed");

    println!("\n  ── Verifying Key (vk) components ──");
    println!("\n  α·G₁  (alpha times G1 generator):");
    print_g1("    vk.alpha_g1 = α·G₁", vk.alpha_g1);
    println!("    Meaning: This binds the proof to α. Used in the pairing");
    println!("    equation as e(α·G₁, β·G₂) = e(α, β) in GT.");

    println!("\n  β·G₂  (beta times G2 generator):");
    print_g2("    vk.beta_g2 = β·G₂", vk.beta_g2);
    println!("    Meaning: Paired with α·G₁ in the verification equation.");
    println!("    e(α·G₁, β·G₂) must equal e(Proof.A, Proof.B) if valid.");

    println!("\n  γ·G₂  (gamma times G2 generator):");
    print_g2("    vk.gamma_g2 = γ·G₂", vk.gamma_g2);
    println!("    Meaning: Used to blind the public input terms in the");
    println!("    verification pairing equation. Enables zero-knowledge.");

    println!("\n  δ·G₂  (delta times G2 generator):");
    print_g2("    vk.delta_g2 = δ·G₂", vk.delta_g2);
    println!("    Meaning: Used in the denominator for proof elements.");
    println!("    Enables the quotient polynomial division: h = (A·B-C)/δ");

    println!("\n  γ⁻¹·(β·aᵢ + α·bᵢ + cᵢ)·G₁  (for each public input i):");
    println!("    vk.gamma_abc_g1: {} elements", vk.gamma_abc_g1.len());
    for (i, elem) in vk.gamma_abc_g1.iter().enumerate() {
        print_g1(
            &format!("    vk.gamma_abc_g1[{}] = γ⁻¹·(β·aᵢ+α·bᵢ+cᵢ)·G₁", i),
            *elem,
        );
    }
    println!("    Meaning: These encode the public input constraints.");
    println!("    For our circuit with 1 public input (c=8):");
    println!("      gamma_abc_g1[0] = γ⁻¹·(β·u₀(τ) + α·v₀(τ) + w₀(τ))·G₁");
    println!("      gamma_abc_g1[1] = γ⁻¹·(β·u₁(τ) + α·v₁(τ) + w₁(τ))·G₁");
    println!("    where u_i, v_i, w_i are the QAP polynomials for column i.");

    println!("\n  ── Proving Key (pk) additional components ──");
    println!("\n  β·G₁  (beta times G1 generator):");
    print_g1("    pk.beta_g1 = β·G₁", pk.beta_g1);
    println!("    Meaning: Used by the prover to construct proof element A.");
    println!("    A = α·G₁ + Σ(r_i · a_i·G₁) + β·G₁ · randomness");

    println!("\n  δ·G₁  (delta times G1 generator):");
    print_g1("    pk.delta_g1 = δ·G₁", pk.delta_g1);
    println!("    Meaning: Used by the prover for randomization and C element.");
    println!("    Proof element C involves division by δ in the exponent.");

    println!("\n  aᵢ·G₁  (A-query: QAP polynomial a_i evaluated at τ, times G₁):");
    println!("    pk.a_query: {} elements", pk.a_query.len());
    for (i, elem) in pk.a_query.iter().enumerate() {
        print_g1(&format!("    pk.a_query[{}] = aᵢ(τ)·G₁", i), *elem);
    }
    println!("    Meaning: These are g^{{a_i(τ)}} where a_i are the QAP");
    println!("    polynomials derived from matrix A. Used to build proof element A.");

    println!("\n  bᵢ·G₁  (B-query in G1: QAP polynomial b_i evaluated at τ, times G₁):");
    println!("    pk.b_g1_query: {} elements", pk.b_g1_query.len());
    for (i, elem) in pk.b_g1_query.iter().enumerate() {
        print_g1(&format!("    pk.b_g1_query[{}] = bᵢ(τ)·G₁", i), *elem);
    }
    println!("    Meaning: These are g^{{b_i(τ)}}. Used to build proof element A and B in G1.");

    println!("\n  bᵢ·G₂  (B-query in G2: QAP polynomial b_i evaluated at τ, times G₂):");
    println!("    pk.b_g2_query: {} elements", pk.b_g2_query.len());
    for (i, elem) in pk.b_g2_query.iter().enumerate() {
        print_g2(&format!("    pk.b_g2_query[{}] = bᵢ(τ)·G₂", i), *elem);
    }
    println!("    Meaning: These are H^{{b_i(τ)}}. Used to build proof element B in G2.");

    println!("\n  hᵢ·G₁  (H-query: quotient polynomial coefficients times G₁):");
    println!("    pk.h_query: {} elements", pk.h_query.len());
    for (i, elem) in pk.h_query.iter().enumerate() {
        print_g1(&format!("    pk.h_query[{}] = hᵢ·G₁", i), *elem);
    }
    println!("    Meaning: These encode h(τ)·G₁ where h(x) = (A(x)·B(x) - C(x)) / Z_H(x)");
    println!("    The prover computes C using these elements.");

    println!("\n  lᵢ·G₁  (L-query: witness-dependent terms times G₁):");
    println!("    pk.l_query: {} elements", pk.l_query.len());
    for (i, elem) in pk.l_query.iter().enumerate() {
        print_g1(&format!("    pk.l_query[{}] = lᵢ·G₁", i), *elem);
    }
    println!("    Meaning: These are δ⁻¹·(β·aᵢ + α·bᵢ + cᵢ)·G₁ for witness variables.");
    println!("    They allow the prover to include witness values in proof element C.");

    // ------------------------------------------------------------------
    // PART 5: Proof Generation
    // ------------------------------------------------------------------
    println!("\n\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PART 5: Proof Generation (Prove)                          │");
    println!("└─────────────────────────────────────────────────────────────┘");

    // The prover generates random scalars r and s for zero-knowledge.
    // These are the "blinding factors" that make the proof hide the witnesses.
    let r: Fr = Fr::rand(&mut setup_rng);
    let s: Fr = Fr::rand(&mut setup_rng);

    print_fr("    r (random blinding factor 1)", r);
    print_fr("    s (random blinding factor 2)", s);

    println!("\n  The proof consists of 3 group elements (A, B, C):");
    println!("\n  Proof element A ∈ G1:");
    println!("    A = g^{{α + r·δ + Σ(r_i · a_i(τ))}}");
    println!("    In group notation: A = α·G₁ + r·δ·G₁ + Σ(r_i · a_i·G₁)");
    println!("    This is a point on the G1 curve.");

    println!("\n  Proof element B ∈ G2:");
    println!("    B = g^{{β + s·δ + Σ(s_i · b_i(τ))}}");
    println!("    In group notation: B = β·G₂ + s·δ·G₂ + Σ(s_i · b_i·G₂)");
    println!("    This is a point on the G2 curve (coordinates in Fq²).");

    println!("\n  Proof element C ∈ G1:");
    println!("    C = g^{{(h(τ) + r·s·δ + r·B_in_G1 + s·A_in_G1) / δ}}");
    println!("    Where h(τ) = (A_QAP(τ)·B_QAP(τ) - C_QAP(τ)) / Z_H(τ)");
    println!("    The division by δ is done in the exponent (multiplication by δ⁻¹).");
    println!("    This is a point on the G1 curve.");

    let proof = <Groth16<Bls12_381> as SNARK<_>>::prove(&pk, circuit.clone(), &mut setup_rng)
        .expect("Groth16 prove failed");

    println!("\n  Generated proof:");
    print_g1("    Proof.A (∈ G1)", proof.a);
    print_g2("    Proof.B (∈ G2)", proof.b);
    print_g1("    Proof.C (∈ G1)", proof.c);

    // ------------------------------------------------------------------
    // PART 6: Proof Verification
    // ------------------------------------------------------------------
    println!("\n\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PART 6: Proof Verification (Verify)                       │");
    println!("└─────────────────────────────────────────────────────────────┘");

    // Preprocess the verifying key for faster verification
    let pvk = PreparedVerifyingKey::<Bls12_381>::from(vk.clone());

    // The prepared VK precomputes:
    //   alpha_g1_beta_g2 = e(α·G₁, β·G₂) ∈ GT
    //   gamma_g2_neg_pc  = -(γ·G₂) prepared for pairing
    //   delta_g2_neg_pc  = -(δ·G₂) prepared for pairing
    println!("\n  Prepared Verifying Key (precomputed for fast verification):");
    println!("    e(α·G₁, β·G₂) ∈ GT = {:?}", pvk.alpha_g1_beta_g2);
    println!("    -γ·G₂ (prepared) = {:?}", pvk.gamma_g2_neg_pc);
    println!("    -δ·G₂ (prepared) = {:?}", pvk.delta_g2_neg_pc);

    // Public input: just c = 8
    let public_input = vec![Fr::from(8u64)];

    println!("\n  Public input (what the verifier knows): c = {}", 8u64);
    println!("  Proof (what the verifier receives): A∈G1, B∈G2, C∈G1");
    println!("  Verifier does NOT know: a=3, b=5, r, s, or any toxic waste");

    // Prepare inputs: compute the public input contribution to the pairing
    let prepared_inputs = Groth16::<Bls12_381>::prepare_inputs(&pvk, &public_input).unwrap();
    println!("\n  Prepared public input contribution:");
    print_g1("    Σ(y_i · γ⁻¹·(β·u_i+α·v_i+w_i)·G₁)", prepared_inputs);

    // The verification equation:
    //
    //   e(A, B) = e(α·G₁, β·G₂) · e(prepared_inputs, γ·G₂) · e(C, δ·G₂)
    //
    // Or equivalently, rearranging to one side:
    //
    //   e(A, B) · e(prepared_inputs, -γ·G₂) · e(C, -δ·G₂) = e(α·G₁, β·G₂)
    //
    // In Miller loop + final exponentiation form:
    //   multi_miller_loop([A, prepared_inputs, C], [B, -γ·G₂, -δ·G₂])
    //     ^final_exp→ GT
    //   must equal e(α·G₁, β·G₂)
    //
    // This single pairing equation verifies:
    //   1. A·B - C is divisible by the vanishing polynomial Z_H (all constraints satisfied)
    //   2. The public inputs match the committed values
    //   3. The proof was generated with the correct α, β, γ, δ (knowledge soundness)
    //   4. No information about a=3, b=5 is leaked (zero-knowledge via r, s blinding)

    println!("\n  ── The Verification Pairing Equation ──");
    println!("\n    e(A, B)  =  e(α·G₁, β·G₂)  ·  e(prepared_inputs, γ·G₂)  ·  e(C, δ·G₂)");
    println!("\n  Where:");
    println!("    A = Proof.a  ∈ G1   (prover's commitment to linear combo)");
    println!("    B = Proof.b  ∈ G2   (prover's commitment to linear combo)");
    println!("    C = Proof.c  ∈ G1   (prover's commitment to quotient poly)");
    println!("    α·G₁ = vk.alpha_g1");
    println!("    β·G₂ = vk.beta_g2");
    println!("    γ·G₂ = vk.gamma_g2");
    println!("    δ·G₂ = vk.delta_g2");
    println!("    prepared_inputs = Σ(public_input_i · vk.gamma_abc_g1[i])");
    println!("\n  This equation holds IF AND ONLY IF:");
    println!("    • The QAP polynomial relation A(x)·B(x) - C(x) is divisible by Z_H(x)");
    println!("    • The public inputs are correctly encoded");
    println!("    • The proof was generated with valid toxic waste");

    // Actually verify
    let verified =
        <Groth16<Bls12_381> as SNARK<_>>::verify_with_processed_vk(&pvk, &public_input, &proof)
            .expect("Groth16 verify failed");

    println!("\n  ── Verification Result ──");
    println!("    Proof verified: {}", verified);

    if verified {
        println!("    ✓ The proof is VALID!");
        println!("    ✓ The prover knows a, b such that a + b = 8");
        println!("    ✓ But the verifier learned NOTHING about a=3 or b=5");
    } else {
        panic!("Verification should have passed!");
    }

    // ------------------------------------------------------------------
    // PART 7: Reject Invalid Input
    // ------------------------------------------------------------------
    println!("\n\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PART 7: Rejecting Invalid Public Inputs                   │");
    println!("└─────────────────────────────────────────────────────────────┘");

    let bad_input = vec![Fr::from(42u64)];
    let should_fail =
        <Groth16<Bls12_381> as SNARK<_>>::verify_with_processed_vk(&pvk, &bad_input, &proof)
            .expect("Groth16 verify failed");

    println!("\n  Trying to verify with wrong public input c = 42:");
    println!("    Verified: {}", should_fail);
    assert!(!should_fail, "Must reject proof with wrong public input!");
    println!("    ✓ Correctly REJECTED — the proof only works for c=8");

    // ------------------------------------------------------------------
    // Summary
    // ------------------------------------------------------------------
    println!("\n\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  SUMMARY: Groth16 Protocol Flow                            │");
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("\n  1. SETUP (trusted):");
    println!("     Sample toxic waste: α, β, γ, δ, τ  (MUST BE DELETED)");
    println!("     Build R1CS → QAP polynomials from circuit");
    println!("     Compute pk = {{α·G₁, β·G₁, δ·G₁, aᵢ·G₁, bᵢ·G₁, bᵢ·G₂, hᵢ·G₁, lᵢ·G₁}}");
    println!("     Compute vk = {{α·G₁, β·G₂, γ·G₂, δ·G₂, γ⁻¹·(... )·G₁}}");
    println!("\n  2. PROVE (prover knows a, b):");
    println!("     Sample random r, s for zero-knowledge");
    println!("     Compute A = g^{{α + r·δ + ...}}  ∈ G1");
    println!("     Compute B = H^{{β + s·δ + ...}}  ∈ G2");
    println!("     Compute C = g^{{h(τ)/δ + r·s·δ + ...}}  ∈ G1");
    println!("     Send proof π = (A, B, C) to verifier");
    println!("\n  3. VERIFY (verifier knows only c and vk):");
    println!("     Check: e(A,B) = e(α·G₁,β·G₂)·e(inputs,γ·G₂)·e(C,δ·G₂)");
    println!("     If true: proof is valid, a+b=c is proven without revealing a,b");
    println!("\n  Key Properties:");
    println!("    • Completeness: honest prover with valid witness → verification passes");
    println!("    • Soundness: no PPT adversary can forge proof for invalid statement");
    println!("    • Zero-Knowledge: verifier learns nothing beyond 'statement is true'");
    println!("    • Size: proof = 3 group elements (~2KB), verification = 2 pairings");

    println!("\n\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  All demonstrations completed successfully! ✓               ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
}

// ============================================================================
// Test: end-to-end Groth16 setup, prove, verify
// ============================================================================
#[test]
fn test_groth16_full_setup_demo() {
    run_groth16_setup_demo();
}

// an end to end example for groth16, using the low-level R1CS API.
//
// Circuit:  y = x² + x + z
//   y — public (instance)
//   x, z — private (witness)
//
// ─── Groth16 Protocol ──────────────────────────────────────────────────────────
// 1. Setup:     circuit-specific CRS → (pk, vk)
// 2. Prove:     build R1CS from circuit + witness → generate proof
// 3. Verify:    check pairing equations using vk + public inputs + proof

#[test]
fn test_groth16() {
    use ark_bls12_381::{Bls12_381, Fr};
    use ark_groth16::{Groth16, PreparedVerifyingKey};
    use ark_r1cs_std::alloc::AllocVar;
    use ark_r1cs_std::eq::EqGadget;
    use ark_r1cs_std::fields::fp::FpVar;
    use ark_r1cs_std::fields::FieldVar;
    use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
    use ark_snark::SNARK;
    use ark_std::rand::{rngs::StdRng, SeedableRng};

    // ─── Circuit Definition ─────────────────────────────────────────────────
    #[derive(Clone)]
    struct SquareCircuit {
        /// Public input: y (= x² + x + z)
        y: Option<Fr>,
        /// Private witness: x
        x: Option<Fr>,
        /// Private witness: z
        z: Option<Fr>,
    }

    // Implement ConstraintSynthesizer — this is how Groth16 knows
    // how to build the R1CS from a concrete assignment.
    impl ConstraintSynthesizer<Fr> for SquareCircuit {
        fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
            // Allocate public input (instance variable)
            let y_var = FpVar::<Fr>::new_input(cs.clone(), || {
                self.y.ok_or(SynthesisError::AssignmentMissing)
            })?;
            // Allocate private witnesses
            let x_var = FpVar::<Fr>::new_witness(cs.clone(), || {
                self.x.ok_or(SynthesisError::AssignmentMissing)
            })?;
            let z_var =
                FpVar::<Fr>::new_witness(cs, || self.z.ok_or(SynthesisError::AssignmentMissing))?;

            // Enforce:  y = x² + x + z   ⇔   x² + x + z - y = 0
            let x_sq = x_var.square()?;
            let lhs = x_sq + x_var + z_var;
            lhs.enforce_equal(&y_var)?;

            Ok(())
        }
    }

    // ─── Test ───────────────────────────────────────────────────────────────
    let mut rng = StdRng::seed_from_u64(0u64);

    // Instance: y = x² + x + z  with x=3, z=1  ⇒  y = 9 + 3 + 1 = 13
    let circuit = SquareCircuit {
        y: Some(Fr::from(13u64)),
        x: Some(Fr::from(3u64)),
        z: Some(Fr::from(1u64)),
    };

    // ========================================================================
    // STEP 1: Setup — generate proving key and verifying key
    // ========================================================================
    // Groth16.circuit_specific_setup runs:
    //   1. Call circuit.generate_constraints() to build the R1CS matrices (A, B, C)
    //   2. Compute QAP polynomials u_i(x), v_i(x), w_i(x) via Lagrange interpolation
    //   3. Sample toxic waste α, β, γ, δ, τ
    //   4. Compute proving key:  g^{α}, g^{β}, g^{δ}, g^{β·u_i(τ)}, g^{α·v_i(τ)}, etc.
    //   5. Compute verifying key:  g^{α}, g^{β}, g^{γ}, g^{δ}, g^{β·v_k(τ)} (for public inputs)
    //
    // CRITICAL: The toxic waste (α, β, γ, δ, τ) must be discarded after setup.
    //           If leaked, fake proofs can be forged.
    println!("Generating proving key (pk) and verifying key (vk)...");
    let (pk, vk) =
        <Groth16<Bls12_381> as SNARK<Fr>>::circuit_specific_setup(circuit.clone(), &mut rng)
            .expect("Groth16 setup failed");
    println!(
        "  ✓ Setup complete (pk size: {}, vk size: {})",
        std::mem::size_of_val(&pk),
        std::mem::size_of_val(&vk)
    );

    // ========================================================================
    // STEP 2: Prove — generate a Groth16 proof
    // ========================================================================
    // Groth16.prove runs:
    //   1. Rebuild R1CS from circuit (same constraints)
    //   2. Compute witness assignment (all wire values, public + private)
    //   3. Compute the three proof elements:
    //      A  = g^{α} · Σ r_i · u_i(τ)
    //      B  = g^{β} · Σ r_i · v_i(τ)
    //      C  = g^{δ^{-1}(Σ r_i · (β·u_i(τ) + α·v_i(τ) + w_i(τ)) + h(τ)·Z_H(τ))}
    //      where r_i are the QAP evaluations at the witness values,
    //      and h(x) is the quotient polynomial h(x) = (A(x)·B(x) - C(x)) / Z_H(x)
    //
    // The proof consists of three group elements: (A, B, C)
    println!("Generating proof...");
    let proof = <Groth16<Bls12_381> as SNARK<_>>::prove(&pk, circuit, &mut rng)
        .expect("Groth16 prove failed");
    println!("  ✓ Proof generated (3 group elements: A, B, C)");

    // ========================================================================
    // STEP 3: Verify — check the proof against public inputs
    // ========================================================================
    // Groth16.verify_with_processed_vk checks the pairing equation:
    //
    //   e(A, B)  =  e(α, β)  ·  e(Π_{i∈pub} g^{u_i(τ)·y_i}, γ)  ·  e(C, δ)
    //
    // where y_i are the public inputs. If the proof is valid, the equation
    // holds iff the QAP polynomial A(x)·B(x) - C(x) is divisible by Z_H(x)
    // at the witness evaluation (i.e., the constraints are satisfied).
    //
    // Preprocess vk for faster verification (pre-computes pairings).
    println!("Verifying proof...");
    let pvk = PreparedVerifyingKey::<Bls12_381>::from(vk);
    let public_input = vec![Fr::from(13u64)];

    let verified =
        <Groth16<Bls12_381> as SNARK<_>>::verify_with_processed_vk(&pvk, &public_input, &proof)
            .expect("Groth16 verify failed");
    assert!(verified, "Groth16 proof verification failed");

    println!("  ✓ Proof verified! y = x² + x + z = 3² + 3 + 1 = 13");
    println!();
    println!("=== Groth16 end-to-end test PASSED ===");

    // ========================================================================
    // BONUS: Reject an invalid proof (wrong public input)
    // ========================================================================
    let bad_input = vec![Fr::from(42u64)]; // Wrong y value
    let should_fail =
        <Groth16<Bls12_381> as SNARK<_>>::verify_with_processed_vk(&pvk, &bad_input, &proof)
            .expect("Groth16 verify failed");
    assert!(!should_fail, "Must reject proof with wrong public input");
    println!("  ✓ Correctly rejects invalid public input");
}

use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::PrimeField;
use ark_groth16::{Groth16, PreparedVerifyingKey};
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::SNARK;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Clone)]
struct SquareCircuit<F: PrimeField> {
    /// Public output: y = x^2 (Groth16 instance column).
    y: Option<F>,
    /// Private witness: x.
    x: Option<F>,
    /// Private witness: z.
    z: Option<F>,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for SquareCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // y is a public input (instance); x stays private (witness).
        let y_var = FpVar::<F>::new_input(cs.clone(), || {
            self.y.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let x_var = FpVar::<F>::new_witness(cs.clone(), || {
            self.x.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let z_var = FpVar::<F>::new_witness(cs.clone(), || {
            self.z.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let x_squared = x_var.square()?;
        let plus_one = x_squared + x_var + z_var;
        plus_one.enforce_equal(&y_var)?;

        Ok(())
    }
}

#[test]
fn test_e2e() {
    // Set up random number generator
    let mut rng = StdRng::seed_from_u64(0u64);

    // Define the circuit for parameters
    // y = 12 (public), x = 3 (private)
    let circuit = SquareCircuit::<Fr> {
        y: Some(Fr::from(13_u64)),
        x: Some(Fr::from(3_u64)),
        z: Some(Fr::from(1_u64)),
    };

    println!("Generating parameters for SquareCircuit...");
    // Generate the proving and verification keys using Groth16
    let (pk, vk) =
        <Groth16<Bls12_381> as SNARK<Fr>>::circuit_specific_setup(circuit.clone(), &mut rng)
            .expect("Failed to generate proving and verification keys");

    // Create a proof
    println!("Generating proof...");
    let proof = <Groth16<Bls12_381> as SNARK<_>>::prove(&pk, circuit, &mut rng)
        .expect("Failed to generate proof");

    // Prepare the verification key (for Groth16, we need to prepare it for verification)
    let pvk = PreparedVerifyingKey::<Bls12_381>::from(vk);

    // Prepare public input (y = 12)
    let public_input = vec![Fr::from(13_u64)];

    // Verify the proof
    println!("Verifying proof...");
    let verified =
        <Groth16<Bls12_381> as SNARK<_>>::verify_with_processed_vk(&pvk, &public_input, &proof)
            .expect("Failed to verify proof");

    assert!(verified, "Proof verification failed");
}

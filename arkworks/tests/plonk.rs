// an end to end example for plonk, using the low-level CS API directly.
//
// ─── Groth16 vs PLONK ─────────────────────────────────────────────────────────
//                  | Groth16                          | PLONK
// ─────────────────┼──────────────────────────────────┼──────────────────────────
// Setup            | Circuit-specific (per circuit)   | Universal (all circuits
//                  |                                  | up to given max degree)
// Proof size       | 3 group elements (~200 B)        | ~10+ elements (~1-2 KB)
// Prover cost      | Faster (1 multi-exp per gate)    | Slower (many poly
//                  |                                  | commits + FFTs)
// Verifier cost    | 3 pairings                       | ~12 pairings
// ─────────────────┴──────────────────────────────────┴──────────────────────────
#[test]
fn test_plonk() {
    use ark_bls12_381::{Bls12_381, Fr};
    use ark_poly::univariate::DensePolynomial;
    use ark_poly::{EvaluationDomain, Evaluations, GeneralEvaluationDomain, Polynomial};
    use ark_poly_commit::kzg10::{
        Commitment, Powers, Proof, Randomness, UniversalParams, VerifierKey, KZG10,
    };
    use ark_std::{borrow::Cow, test_rng, One, Zero};

    type UniPoly = DensePolynomial<Fr>;
    type KZG = KZG10<Bls12_381, UniPoly>;

    let rng = &mut test_rng();

    // ============================================================
    // STEP 1: Domain  (smallest power of 2 that fits the circuit)
    // ============================================================
    let n = 4;
    let domain = GeneralEvaluationDomain::<Fr>::new(n).unwrap();

    // ============================================================
    // STEP 2: Circuit description — selector polynomials
    // ============================================================
    // PLONK gate: q_M·a·b + q_L·a + q_R·b + q_O·c + q_C = 0
    //
    // Multiplication gate a·b = c:
    //   q_M = 1, q_O = -1  at the first gate, all 0 elsewhere
    let q_m: UniPoly = Evaluations::from_vec_and_domain(
        vec![Fr::one(), Fr::zero(), Fr::zero(), Fr::zero()],
        domain,
    )
    .interpolate();

    let _q_l: UniPoly = Evaluations::from_vec_and_domain(vec![Fr::zero(); 4], domain).interpolate();
    let _q_r: UniPoly = Evaluations::from_vec_and_domain(vec![Fr::zero(); 4], domain).interpolate();
    let q_o: UniPoly = Evaluations::from_vec_and_domain(
        vec![-Fr::one(), Fr::zero(), Fr::zero(), Fr::zero()],
        domain,
    )
    .interpolate();
    let _q_c: UniPoly = Evaluations::from_vec_and_domain(vec![Fr::zero(); 4], domain).interpolate();

    // ============================================================
    // STEP 3: Trusted setup — KZG CRS (powers of τ)
    // ============================================================
    let max_degree = 8_usize;
    let pp: UniversalParams<Bls12_381> =
        KZG::setup(max_degree, false, rng).expect("KZG setup failed");

    let supported_degree = 6_usize;
    let powers_of_g: Vec<ark_bls12_381::G1Affine> = pp.powers_of_g[..=supported_degree].to_vec();
    let powers_of_gamma_g: Vec<ark_bls12_381::G1Affine> = (0..=supported_degree)
        .map(|i| pp.powers_of_gamma_g[&i])
        .collect();
    let powers: Powers<'_, Bls12_381> = Powers {
        powers_of_g: Cow::Owned(powers_of_g),
        powers_of_gamma_g: Cow::Owned(powers_of_gamma_g),
    };
    let vk: VerifierKey<Bls12_381> = VerifierKey {
        g: pp.powers_of_g[0],
        gamma_g: pp.powers_of_gamma_g[&0],
        h: pp.h,
        beta_h: pp.beta_h,
        prepared_h: pp.prepared_h.clone(),
        prepared_beta_h: pp.prepared_beta_h.clone(),
    };

    // ============================================================
    // STEP 4: Prover — witness assignment & interpolation
    // ============================================================
    // Private: a = 2, b = 3
    // Public:  c = 6
    let a_poly: UniPoly = Evaluations::from_vec_and_domain(
        vec![Fr::from(2u64), Fr::zero(), Fr::zero(), Fr::zero()],
        domain,
    )
    .interpolate();
    let b_poly: UniPoly = Evaluations::from_vec_and_domain(
        vec![Fr::from(3u64), Fr::zero(), Fr::zero(), Fr::zero()],
        domain,
    )
    .interpolate();
    let c_poly: UniPoly = Evaluations::from_vec_and_domain(
        vec![Fr::from(6u64), Fr::zero(), Fr::zero(), Fr::zero()],
        domain,
    )
    .interpolate();

    // Sanity: the constraint must be satisfied at every domain point
    for i in 0..n {
        let x = domain.element(i);
        let gate = q_m.evaluate(&x) * a_poly.evaluate(&x) * b_poly.evaluate(&x)
            + q_o.evaluate(&x) * c_poly.evaluate(&x);
        assert_eq!(gate, Fr::zero(), "gate constraint must hold at H");
    }
    println!("✓ Gate constraint satisfied at all {} domain points", n);

    // ============================================================
    // STEP 5: Prover — commit to witness polynomials
    // ============================================================
    let (comm_a, rand_a): (Commitment<Bls12_381>, Randomness<Fr, UniPoly>) = KZG::commit(
        &powers,
        &a_poly,
        None,
        None::<&mut dyn ark_std::rand::RngCore>,
    )
    .unwrap();
    let (comm_b, rand_b): (Commitment<Bls12_381>, Randomness<Fr, UniPoly>) = KZG::commit(
        &powers,
        &b_poly,
        None,
        None::<&mut dyn ark_std::rand::RngCore>,
    )
    .unwrap();
    let (comm_c, rand_c): (Commitment<Bls12_381>, Randomness<Fr, UniPoly>) = KZG::commit(
        &powers,
        &c_poly,
        None,
        None::<&mut dyn ark_std::rand::RngCore>,
    )
    .unwrap();
    println!("✓ Witness polynomials committed");

    // ============================================================
    // STEP 6: Prover — quotient polynomial  t(x) = f(x) / Z_H(x)
    // ============================================================
    // f(x) = q_M·a·b + q_L·a + q_R·b + q_O·c + q_C
    // Z_H(x) = x^n - 1
    let ab: UniPoly = &a_poly * &b_poly;
    let mut numerator: UniPoly = &q_m * &ab;
    numerator = &numerator + &(&q_o * &c_poly);

    let (t_poly, rem) = numerator.divide_by_vanishing_poly(domain).unwrap();
    assert!(rem.is_zero(), "Numerator must be divisible by Z_H");
    println!("✓ Quotient polynomial computed (deg {})", t_poly.degree());

    let (comm_t, rand_t): (Commitment<Bls12_381>, Randomness<Fr, UniPoly>) = KZG::commit(
        &powers,
        &t_poly,
        None,
        None::<&mut dyn ark_std::rand::RngCore>,
    )
    .unwrap();

    // ============================================================
    // STEP 7: Prover — evaluate & open at random challenge ζ
    // ============================================================
    let zeta: Fr = Fr::from(42u64);

    let open_poly =
        |poly: &UniPoly, rand: &Randomness<Fr, UniPoly>, z: Fr| -> (Fr, Proof<Bls12_381>) {
            let v = poly.evaluate(&z);
            let (w_poly, _) =
                KZG::compute_witness_polynomial(poly, z, rand).expect("compute witness failed");
            let (comm_w, _) = KZG::commit(
                &powers,
                &w_poly,
                None,
                None::<&mut dyn ark_std::rand::RngCore>,
            )
            .expect("commit witness failed");
            let proof = Proof {
                w: comm_w.0,
                random_v: None,
            };
            (v, proof)
        };

    let (a_zeta, proof_a) = open_poly(&a_poly, &rand_a, zeta);
    let (b_zeta, proof_b) = open_poly(&b_poly, &rand_b, zeta);
    let (c_zeta, proof_c) = open_poly(&c_poly, &rand_c, zeta);
    let (t_zeta, proof_t) = open_poly(&t_poly, &rand_t, zeta);

    // ============================================================
    // STEP 8: Verifier — verify proof
    // ============================================================

    // 8a. Verify KZG opening proofs
    assert!(KZG::check(&vk, &comm_a, zeta, a_zeta, &proof_a).unwrap());
    assert!(KZG::check(&vk, &comm_b, zeta, b_zeta, &proof_b).unwrap());
    assert!(KZG::check(&vk, &comm_c, zeta, c_zeta, &proof_c).unwrap());
    assert!(KZG::check(&vk, &comm_t, zeta, t_zeta, &proof_t).unwrap());
    println!("✓ All KZG opening proofs verified");

    // 8b. Evaluate public polynomials at ζ
    let q_m_zeta = q_m.evaluate(&zeta);
    let q_o_zeta = q_o.evaluate(&zeta);
    let zh_zeta = domain.evaluate_vanishing_polynomial(zeta);

    // 8c. Check PLONK gate equation:  q_M·a·b + q_O·c = Z_H·t
    let lhs = q_m_zeta * a_zeta * b_zeta + q_o_zeta * c_zeta;
    let rhs = zh_zeta * t_zeta;
    assert_eq!(lhs, rhs, "PLONK gate equation must hold at ζ");

    println!("✓ PLONK gate equation holds: q_M(ζ)·a·b + q_O(ζ)·c = Z_H(ζ)·t(ζ)");
    println!("  a(ζ) = {}, b(ζ) = {}, c(ζ) = {}", a_zeta, b_zeta, c_zeta);
    println!("  a·b = {}", a_zeta * b_zeta);
    println!();
    println!("=== PLONK proof end-to-end test PASSED ===");
}

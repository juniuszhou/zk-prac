// PLONKish table: how PLONK represents constraints through gates and copy constraints.
//
// ─── Core Concepts ─────────────────────────────────────────────────────────
//
// 1. PLONKish Table
//    ┌─────┬─────┬─────┬──────┬──────┬──────┬──────┬──────┐
//    │  a  │  b  │  c  │  q_M │  q_L │  q_R │  q_O │  q_C │   ← columns
//    ├─────┼─────┼─────┼──────┼──────┼──────┼──────┼──────┤
//    │  x  │  y  │ sum │  0   │  1   │  1   │  -1  │  0   │   ← row 0 (addition)
//    │ sum │  z  │ out │  1   │  0   │  0   │  -1  │  0   │   ← row 1 (multiply)
//    └─────┴─────┴─────┴──────┴──────┴──────┴──────┴──────┘
//
//    Each row = one gate. Selectors (q_*) define the gate type.
//    Gate equation (per row):  q_M·a·b + q_L·a + q_R·b + q_O·c + q_C = 0
//
//    Addition: q_L=1, q_R=1, q_O=-1  ⇒  a + b - c = 0
//    Multiply: q_M=1, q_O=-1         ⇒  a·b - c = 0
//
// 2. Copy Constraints (Permutation Argument)
//    When c[row0] == a[row1], those two cells must hold the same value.
//    PLONK defines a permutation σ over all cell positions (col, row):
//    positions in the same cycle must have equal values.
//    An accumulator polynomial z(x) checks that σ is satisfied.

#[test]
fn test_plonk_table() {
    use ark_bls12_381::{Bls12_381, Fr};
    use ark_poly::univariate::DensePolynomial;
    use ark_poly::{EvaluationDomain, Evaluations, GeneralEvaluationDomain, Polynomial};
    use ark_poly_commit::kzg10::{Powers, Proof, Randomness, VerifierKey, KZG10};
    use ark_std::{borrow::Cow, test_rng, One, Zero};

    type UniPoly = DensePolynomial<Fr>;
    type KZG = KZG10<Bls12_381, UniPoly>;

    // ========================================================================
    // Circuit:  out = (x + y) × z
    //   x=2, y=3, z=4  ⇒  out = (2+3)×4 = 20
    //
    // Gate 0 (addition):  c = a + b   →  q_L=1, q_R=1, q_O=-1
    // Gate 1 (multiply):  c = a × b   →  q_M=1, q_O=-1
    // Copy constraint:  c[0] == a[1]  (sum fed forward)
    // ========================================================================
    let x = Fr::from(2u64);
    let y = Fr::from(3u64);
    let z = Fr::from(4u64);
    let sum = x + y;
    let out = sum * z;

    // ─── Step 1: Build the PLONKish Table ──────────────────────────────────
    // Number of gates (rows). Use next power of 2 for FFT domain.
    let n = 4_usize; // 2 gates + 2 dummy rows

    // Selector columns — these define the circuit
    let q_m = vec![Fr::zero(), Fr::one(), Fr::zero(), Fr::zero()];
    let q_l = vec![Fr::one(), Fr::zero(), Fr::zero(), Fr::zero()];
    let q_r = vec![Fr::one(), Fr::zero(), Fr::zero(), Fr::zero()];
    let q_o = vec![-Fr::one(), -Fr::one(), Fr::zero(), Fr::zero()];
    let q_c = vec![Fr::zero(), Fr::zero(), Fr::zero(), Fr::zero()];

    // Wire columns — assigned by the prover
    let a = vec![x, sum, Fr::zero(), Fr::zero()];
    let b = vec![y, z, Fr::zero(), Fr::zero()];
    let c = vec![sum, out, Fr::zero(), Fr::zero()];
    //                ^^^^  ^^^
    //           c[0]==a[1]   = 5 (copy constraint)

    // ─── Step 2: Verify Gate Equation Row by Row ───────────────────────────
    for row in 0..n {
        let gate = q_m[row] * a[row] * b[row]
            + q_l[row] * a[row]
            + q_r[row] * b[row]
            + q_o[row] * c[row]
            + q_c[row];
        assert_eq!(gate, Fr::zero(), "gate equation failed at row {row}");
    }
    println!("✓ Gate equation holds for all {n} rows");

    // ─── Step 3: Copy Constraints (Permutation) ────────────────────────────
    //
    // PLONK assigns each cell position (col, row) a label, then defines a
    // permutation σ that groups equal cells into cycles. For our circuit:
    //
    //   σ: (2,0) → (0,1)   meaning c[0] and a[1] are in the same cycle
    //
    // Simplified representation: assign a group ID to each cell; cells with
    // the same group ID must have equal values.
    let cols = 3_usize;
    // perm[col][row] = group_id
    let mut perm: Vec<Vec<usize>> = (0..cols)
        .map(|c| (0..n).map(|r| c * n + r).collect())
        .collect();

    // Merge: c[0] and a[1] are in the same group
    let old = perm[2][0]; // group of c[0]
    let new = perm[0][1]; // group of a[1]
    for col in 0..cols {
        for row in 0..n {
            if perm[col][row] == old {
                perm[col][row] = new;
            }
        }
    }

    // Now verify: all cells in the same group must have equal values
    let mut groups: Vec<Vec<(usize, usize)>> = vec![vec![]; n * cols];
    for col in 0..cols {
        for row in 0..n {
            groups[perm[col][row]].push((col, row));
        }
    }
    for group in &groups {
        if group.len() <= 1 {
            continue;
        }
        let values: Vec<Fr> = group
            .iter()
            .map(|&(col, row)| match col {
                0 => a[row],
                1 => b[row],
                _ => c[row],
            })
            .collect();
        for v in &values {
            assert_eq!(*v, values[0], "permutation group mismatch");
        }
    }
    println!("✓ Copy constraints verified (c[0] == a[1] == {sum})");

    // ─── Step 4: Interpolate into Polynomials ──────────────────────────────
    // Convert each column from evaluations (at domain points) to coefficient
    // form. These polynomials encode the entire computation.
    let domain = GeneralEvaluationDomain::<Fr>::new(n).unwrap();

    let a_poly: UniPoly = Evaluations::from_vec_and_domain(a.clone(), domain).interpolate();
    let b_poly: UniPoly = Evaluations::from_vec_and_domain(b.clone(), domain).interpolate();
    let c_poly: UniPoly = Evaluations::from_vec_and_domain(c.clone(), domain).interpolate();
    let q_m_poly: UniPoly = Evaluations::from_vec_and_domain(q_m.clone(), domain).interpolate();
    let q_l_poly: UniPoly = Evaluations::from_vec_and_domain(q_l.clone(), domain).interpolate();
    let q_r_poly: UniPoly = Evaluations::from_vec_and_domain(q_r.clone(), domain).interpolate();
    let q_o_poly: UniPoly = Evaluations::from_vec_and_domain(q_o.clone(), domain).interpolate();
    let q_c_poly: UniPoly = Evaluations::from_vec_and_domain(q_c.clone(), domain).interpolate();

    println!("✓ Wire/selector polynomials interpolated");

    // ─── Step 5: Compute Quotient Polynomial ───────────────────────────────
    // f(x) = q_M·a·b + q_L·a + q_R·b + q_O·c + q_C
    // t(x) = f(x) / Z_H(x)   (where Z_H(x) = xⁿ - 1)
    let ab: UniPoly = &a_poly * &b_poly;
    let mut numerator: UniPoly = &q_m_poly * &ab;
    numerator = &numerator + &(&q_l_poly * &a_poly);
    numerator = &numerator + &(&q_r_poly * &b_poly);
    numerator = &numerator + &(&q_o_poly * &c_poly);
    numerator = &numerator + &q_c_poly;

    let (t_poly, rem) = numerator.divide_by_vanishing_poly(domain).unwrap();
    assert!(rem.is_zero(), "numerator must be divisible by Z_H");
    println!("✓ Quotient t(x) computed (deg {})", t_poly.degree());

    // ─── Step 6: KZG Commitments ───────────────────────────────────────────
    let rng = &mut test_rng();
    let pp = KZG::setup(10, false, rng).unwrap();

    let powers_of_g: Vec<ark_bls12_381::G1Affine> = pp.powers_of_g[..=8].to_vec();
    let powers_of_gamma_g: Vec<ark_bls12_381::G1Affine> =
        (0..=8).map(|i| pp.powers_of_gamma_g[&i]).collect();
    let powers: Powers<'_, Bls12_381> = Powers {
        powers_of_g: Cow::Owned(powers_of_g.clone()),
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

    let open = |poly: &UniPoly, rand: &Randomness<Fr, UniPoly>, z: Fr| -> (Fr, Proof<Bls12_381>) {
        let v = poly.evaluate(&z);
        let (w, _) = KZG::compute_witness_polynomial(poly, z, rand).unwrap();
        let (cw, _) =
            KZG::commit(&powers, &w, None, None::<&mut dyn ark_std::rand::RngCore>).unwrap();
        (
            v,
            Proof {
                w: cw.0,
                random_v: None,
            },
        )
    };

    let (comm_a, rand_a) = KZG::commit(&powers, &a_poly, None, None).unwrap();
    let (comm_b, rand_b) = KZG::commit(&powers, &b_poly, None, None).unwrap();
    let (comm_c, rand_c) = KZG::commit(&powers, &c_poly, None, None).unwrap();
    let (comm_t, rand_t) = KZG::commit(&powers, &t_poly, None, None).unwrap();
    println!("✓ Witnesses committed via KZG");

    // ─── Step 7: Open at Random Challenge ──────────────────────────────────
    let zeta = Fr::from(42u64);

    let (a_zeta, p_a) = open(&a_poly, &rand_a, zeta);
    let (b_zeta, p_b) = open(&b_poly, &rand_b, zeta);
    let (c_zeta, p_c) = open(&c_poly, &rand_c, zeta);
    let (t_zeta, p_t) = open(&t_poly, &rand_t, zeta);

    // ─── Step 8: Verification ──────────────────────────────────────────────
    // 8a. KZG openings
    for (name, c, v, p) in [
        ("a", &comm_a, a_zeta, &p_a),
        ("b", &comm_b, b_zeta, &p_b),
        ("c", &comm_c, c_zeta, &p_c),
        ("t", &comm_t, t_zeta, &p_t),
    ] {
        assert!(
            KZG::check(&vk, c, zeta, v, p).unwrap(),
            "{name} opening failed"
        );
    }
    println!("✓ All KZG openings verified");

    // 8b. PLONK gate equation at ζ
    let zh_zeta = domain.evaluate_vanishing_polynomial(zeta);
    let q_m_zeta = q_m_poly.evaluate(&zeta);
    let q_l_zeta = q_l_poly.evaluate(&zeta);
    let q_r_zeta = q_r_poly.evaluate(&zeta);
    let q_o_zeta = q_o_poly.evaluate(&zeta);
    let q_c_zeta = q_c_poly.evaluate(&zeta);

    let lhs = q_m_zeta * a_zeta * b_zeta
        + q_l_zeta * a_zeta
        + q_r_zeta * b_zeta
        + q_o_zeta * c_zeta
        + q_c_zeta;
    let rhs = zh_zeta * t_zeta;
    assert_eq!(lhs, rhs, "PLONK gate equation must hold at ζ");

    // NOTE: In real PLONK, copy constraints are enforced by the permutation
    // argument, which uses an accumulator polynomial z(x) to check that cells
    // in the same permutation cycle have equal values across ALL domain points.
    // This requires the permutation challenges β, γ (from Fiat-Shamir) and a
    // separate quotient + opening. We verified the copy constraint at the
    // evaluation level in Step 3; a full PLONK would additionally embed the
    // permutation check into the proof.

    println!("✓ Verification complete: (x+y)×z = ({x}+{y})×{z} = {out}");
    println!();
    println!("=== PLONKish table end-to-end test PASSED ===");
}

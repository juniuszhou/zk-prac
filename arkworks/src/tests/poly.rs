use ark_bls12_381::Fr;
use ark_ff::{BigInteger, Field, One, PrimeField, UniformRand, Zero};
use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_poly::Polynomial;
use num_bigint::BigUint;
use rand::{rngs::StdRng, SeedableRng};

#[test]
fn test_prime_field() {
    let mut rng = StdRng::seed_from_u64(0u64);
    let a = Fr::rand(&mut rng);
    println!("Random field element a: {:?}", a);
    // We can access the prime modulus associated with `F`:
    let modulus = <Fr as PrimeField>::MODULUS;
    assert_eq!(a.pow(&modulus), a); // the Euler-Fermat theorem tells us: a^{p-1} = 1 mod p

    // We can convert field elements to integers in the range [0, MODULUS - 1]:
    let one: BigUint = Fr::one().into();
    assert_eq!(one, BigUint::one());

    // We can construct field elements from an arbitrary sequence of bytes:
    let n = Fr::from_le_bytes_mod_order(&modulus.to_bytes_le());
    assert_eq!(n, Fr::zero());
}

#[test]
fn test_dense_uv_polynomial() {
    let mut rng = StdRng::seed_from_u64(0u64);
    let coeffs = (0..5).map(|_| Fr::rand(&mut rng)).collect::<Vec<_>>();
    let poly = DensePolynomial::from_coefficients_vec(coeffs.clone());

    // Evaluate the polynomial at a random point
    let x = Fr::rand(&mut rng);
    let y = poly.evaluate(&x);

    // Verify that the evaluation is correct
    let mut expected_y = Fr::zero();
    for (i, coeff) in coeffs.iter().enumerate() {
        expected_y += *coeff * x.pow(&[i as u64]);
    }
    assert_eq!(y, expected_y);
}

#[test]
fn test_r1cs_to_qap() {
    use ark_ff::One;
    use ark_poly::{EvaluationDomain, Evaluations, Polynomial, Radix2EvaluationDomain};
    use ark_relations::r1cs::{ConstraintSystem, LinearCombination, Variable};

    // ---------------------------------------------------------------
    // 1. Build a circuit: x^3 + x + 5 = out  (Vitalik's R1CS example)
    //    using the low-level CS API directly.
    //    Variables: ONE (implicit), out (public), x, sym_1, y, sym_2 (witnesses)
    //    Column layout:
    //      0: ONE
    //      1: out           (Instance(1))
    //      2: x             (Witness(0))
    //      3: sym_1         (Witness(1))
    //      4: y             (Witness(2))
    //      5: sym_2         (Witness(3))
    // ---------------------------------------------------------------
    let cs = ConstraintSystem::<Fr>::new_ref();
    let three = Fr::from(3u8);
    let five = Fr::from(5u8);
    let nine = Fr::from(9u8);
    let thirty_five = Fr::from(35u8);

    let out = cs
        .new_input_variable(|| Ok(thirty_five))
        .expect("alloc out");
    let x = cs.new_witness_variable(|| Ok(three)).expect("alloc x");
    let sym_1 = cs.new_witness_variable(|| Ok(nine)).expect("alloc sym_1");
    let y = cs
        .new_witness_variable(|| Ok(nine * three))
        .expect("alloc y");
    let sym_2 = cs
        .new_witness_variable(|| Ok(nine * three + three))
        .expect("alloc sym_2");

    // Constraint 0: x * x = sym_1
    cs.enforce_constraint(
        LinearCombination::from(x),
        LinearCombination::from(x),
        LinearCombination::from(sym_1),
    )
    .expect("enforce constraint 0");
    // Constraint 1: sym_1 * x = y
    cs.enforce_constraint(
        LinearCombination::from(sym_1),
        LinearCombination::from(x),
        LinearCombination::from(y),
    )
    .expect("enforce constraint 1");
    // Constraint 2: (y + x) * 1 = sym_2
    cs.enforce_constraint(
        LinearCombination::from(y) + x,
        LinearCombination::from(Variable::One),
        LinearCombination::from(sym_2),
    )
    .expect("enforce constraint 2");
    // Constraint 3: (sym_2 + 5) * 1 = out
    cs.enforce_constraint(
        LinearCombination::from(sym_2) + (five, Variable::One),
        LinearCombination::from(Variable::One),
        LinearCombination::from(out),
    )
    .expect("enforce constraint 3");

    cs.finalize();
    assert!(cs.is_satisfied().unwrap());

    // ---------------------------------------------------------------
    // 2. Extract R1CS sparse matrices  (Vec<Vec<(coeff, col_idx)>>)
    // ---------------------------------------------------------------
    let matrices = cs.to_matrices().expect("matrices constructed");
    let m = matrices.num_constraints; // number of constraints (rows)
    let num_instance = matrices.num_instance_variables; // includes ONE
    let num_witness = matrices.num_witness_variables;
    let n = num_instance + num_witness; // total variables (columns)

    println!(
        "R1CS: {} constraints × {} cols (ONE + {} instance + {} witness)",
        m,
        n,
        num_instance - 1,
        num_witness
    );

    // ---------------------------------------------------------------
    // 3. Convert sparse matrices to dense m × n form
    // ---------------------------------------------------------------
    let mut a_dense = vec![vec![Fr::zero(); n]; m];
    let mut b_dense = vec![vec![Fr::zero(); n]; m];
    let mut c_dense = vec![vec![Fr::zero(); n]; m];

    for (i, row) in matrices.a.iter().enumerate() {
        for &(coeff, col) in row {
            a_dense[i][col] = coeff;
        }
    }
    for (i, row) in matrices.b.iter().enumerate() {
        for &(coeff, col) in row {
            b_dense[i][col] = coeff;
        }
    }
    for (i, row) in matrices.c.iter().enumerate() {
        for &(coeff, col) in row {
            c_dense[i][col] = coeff;
        }
    }

    // ---------------------------------------------------------------
    // 4. R1CS → QAP  (Lagrange interpolation over a power-of-2 domain)
    //    For each column j, interpolate A_j, B_j, C_j so that
    //      A_j(ω^i) = A[i][j],  same for B, C,  for all rows i.
    // ---------------------------------------------------------------
    let domain_sz = m.next_power_of_two();
    let domain = Radix2EvaluationDomain::<Fr>::new(domain_sz)
        .expect("domain must be constructible (check 2-adicity of Fr)");

    let mut a_polys: Vec<DensePolynomial<Fr>> = Vec::with_capacity(n);
    let mut b_polys: Vec<DensePolynomial<Fr>> = Vec::with_capacity(n);
    let mut c_polys: Vec<DensePolynomial<Fr>> = Vec::with_capacity(n);

    for j in 0..n {
        let fill = |dense: &[Vec<Fr>]| {
            let mut evals = vec![Fr::zero(); domain.size()];
            for i in 0..m {
                evals[i] = dense[i][j];
            }
            evals
        };
        a_polys.push(Evaluations::from_vec_and_domain(fill(&a_dense), domain).interpolate());
        b_polys.push(Evaluations::from_vec_and_domain(fill(&b_dense), domain).interpolate());
        c_polys.push(Evaluations::from_vec_and_domain(fill(&c_dense), domain).interpolate());
    }

    println!(
        "==== QAP: {:?} a polys, {:?} b polys, {:?} c polys",
        a_polys, b_polys, c_polys,
    );

    // ---------------------------------------------------------------
    // 5. Verify the QAP property at every constraint point.
    //    For a valid vector z = (1, out, x, sym_1, y, sym_2):
    //      P(x) = (Σ A_j(x)·z_j) · (Σ B_j(x)·z_j) − Σ C_j(x)·z_j
    //    must vanish at every domain element ωⁱ for i = 0 .. m-1.
    // ---------------------------------------------------------------
    let mut z = vec![Fr::zero(); n];
    z[0] = Fr::one(); // ONE
    z[1] = thirty_five; // out
    let wo = num_instance; // witness offset
    z[wo] = three;
    z[wo + 1] = nine;
    z[wo + 2] = nine * three;
    z[wo + 3] = nine * three + three;

    for i in 0..m {
        let pt = domain.element(i);
        let mut a = Fr::zero();
        let mut b = Fr::zero();
        let mut c = Fr::zero();
        for j in 0..n {
            a += a_polys[j].evaluate(&pt) * z[j];
            b += b_polys[j].evaluate(&pt) * z[j];
            c += c_polys[j].evaluate(&pt) * z[j];
        }
        let p = a * b - c;
        assert!(
            p.is_zero(),
            "QAP fails at constraint {}: P({:?}) = {:?}",
            i,
            pt,
            p
        );
    }
    println!(
        "QAP verified: P(x) vanishes on all {} constraint-domain points",
        m
    );
}

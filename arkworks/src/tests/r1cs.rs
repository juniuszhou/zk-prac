use ark_bls12_381::Fr;
use ark_ff::Zero;
use ark_relations::r1cs::{ConstraintMatrices, ConstraintSystem, LinearCombination, Variable};

#[test]
fn test_r1cs() {
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

    // R1CS constraint system
    let r1cs = ConstraintSystem::<Fr>::new_ref();

    let three: Fr = 3u8.into();
    let five: Fr = 5u8.into();
    let nine: Fr = 9u8.into();
    let thirty_five: Fr = 35u8.into();

    // each column is a variable, and each row is a constraint.
    let out = r1cs
        .new_input_variable(|| Ok(thirty_five))
        .expect("alloc out");
    let x = r1cs.new_witness_variable(|| Ok(three)).expect("alloc x");
    let sym_1 = r1cs.new_witness_variable(|| Ok(nine)).expect("alloc sym_1");
    let y = r1cs
        .new_witness_variable(|| Ok(nine * three))
        .expect("alloc y");
    let sym_2 = r1cs
        .new_witness_variable(|| Ok(nine * three + three))
        .expect("alloc sym_2");

    // Constraint 0: x * x = sym_1
    r1cs.enforce_constraint(
        LinearCombination::from(x),
        LinearCombination::from(x),
        LinearCombination::from(sym_1),
    )
    .expect("enforce constraint 0");
    // Constraint 1: sym_1 * x = y
    r1cs.enforce_constraint(
        LinearCombination::from(sym_1),
        LinearCombination::from(x),
        LinearCombination::from(y),
    )
    .expect("enforce constraint 1");
    // Constraint 2: (y + x) * 1 = sym_2
    r1cs.enforce_constraint(
        LinearCombination::from(y) + x,
        LinearCombination::from(Variable::One),
        LinearCombination::from(sym_2),
    )
    .expect("enforce constraint 2");
    // Constraint 3: (sym_2 + 5) * 1 = out
    r1cs.enforce_constraint(
        LinearCombination::from(sym_2) + (five, Variable::One),
        LinearCombination::from(Variable::One),
        LinearCombination::from(out),
    )
    .expect("enforce constraint 3");

    r1cs.finalize();
    assert!(r1cs.is_satisfied().unwrap());

    // ---------------------------------------------------------------
    // 2. Extract R1CS sparse matrices  (Vec<Vec<(coeff, col_idx)>>)
    // ---------------------------------------------------------------
    // it is the result of all constraints enforced, and the matrices are in sparse form.
    let matrices: ConstraintMatrices<Fr> = r1cs.to_matrices().expect("matrices constructed");

    let rows = matrices.num_constraints; // number of constraints (rows)
    let num_instance = matrices.num_instance_variables; // includes ONE
    let num_witness = matrices.num_witness_variables;
    let columns = num_instance + num_witness; // total variables (columns)

    println!(
        "R1CS: {} constraints × {} cols (ONE + {} instance + {} witness)",
        rows,
        columns,
        num_instance - 1,
        num_witness
    );

    // ---------------------------------------------------------------
    // 3. Convert sparse matrices to dense m × n form
    // ---------------------------------------------------------------
    let mut a_dense = vec![vec![Fr::zero(); columns]; rows];
    let mut b_dense = vec![vec![Fr::zero(); columns]; rows];
    let mut c_dense = vec![vec![Fr::zero(); columns]; rows];

    // to dense format. after we get the coefficients matrix.
    // then we can get the polynomial representation of the constraints. and go to next step: QAP.

    for (i, row) in matrices.a.iter().enumerate() {
        // for each row, which is a vector of (coeff, col_idx) pairs. that's why it's called sparse matrix representation.
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
}

// This binary demonstrates the Quadratic Arithmetic Program (QAP) transformation
// from a Rank-1 Constraint System (R1CS) for the simple constraint: a + b = c.
//
// We consider the witness vector as [one, a, b, c] where 'one' is the constant 1.
//
// The R1CS for the constraint a + b - c = 0 can be written as:
//   A * B = C
// where
//   A = [0, 1, 1, 0]   -> computes (a + b)
//   B = [1, 0, 0, 0]   -> computes 1
//   C = [0, 0, 0, 1]   -> computes c
//
// We then convert this R1CS to a QAP by defining polynomials for each column
// such that at the root of the target polynomial T(x) (chosen as x - r),
// the polynomial evaluates to the corresponding coefficient in the R1CS.
//
// We choose a random point r = 5 for the target polynomial T(x) = x - 5.
//
// For each column i (0..3), we define polynomials U_i(x), V_i(x), W_i(x) as constants:
//   U_i(x) = A[0][i]
//   V_i(x) = B[0][i]
//   W_i(x) = C[0][i]
//
// Then, the polynomial U(x) = sum_i w_i * U_i(x), similarly for V(x) and W(x).
//
// The QAP checks that U(x)*V(x) - W(x) = H(x)*T(x) for some polynomial H(x).
//
// In this example, with one constraint, H(x) is a constant.

fn main() {
    // Define the R1CS matrices for one constraint.
    // Each row corresponds to a constraint, each column to a witness component.
    let a_matrix = vec![vec![0, 1, 1, 0]]; // A matrix
    let b_matrix = vec![vec![1, 0, 0, 0]]; // B matrix
    let c_matrix = vec![vec![0, 0, 0, 1]]; // C matrix

    // Witness vector: [one, a, b, c]
    // We choose a=2, b=3, c=5 (so that a+b=c holds).
    let witness = [1, 2, 3, 5];

    // Choose a random point for the target polynomial T(x) = x - r
    let r = 5;
    // T(x) = -r + 1*x
    let t_coeffs = [-r, 1];

    // For each column, the polynomials are constants (degree 0) equal to the
    // corresponding entry in the R1CS matrices (since we have only one constraint).
    let u_coeffs = a_matrix[0].clone(); // [0, 1, 1, 0]
    let v_coeffs = b_matrix[0].clone(); // [1, 0, 0, 0]
    let w_coeffs = c_matrix[0].clone(); // [0, 0, 0, 1]

    // Evaluate the polynomials at x = r (since T(r)=0, we evaluate at the root).
    // For constant polynomials, the evaluation is just the constant.
    let u_at_r = u_coeffs
        .iter()
        .enumerate()
        .map(|(i, &coeff)| witness[i] as i64 * coeff as i64)
        .sum::<i64>();
    let v_at_r = v_coeffs
        .iter()
        .enumerate()
        .map(|(i, &coeff)| witness[i] as i64 * coeff as i64)
        .sum::<i64>();
    let w_at_r = w_coeffs
        .iter()
        .enumerate()
        .map(|(i, &coeff)| witness[i] as i64 * coeff as i64)
        .sum::<i64>();

    // Check the QAP condition: U(r)*V(r) - W(r) should be 0 because T(r)=0.
    let left = u_at_r * v_at_r - w_at_r;
    println!("For witness a={}, b={}, c={}:", witness[1], witness[2], witness[3]);
    println!("U(r) = {}", u_at_r);
    println!("V(r) = {}", v_at_r);
    println!("W(r) = {}", w_at_r);
    println!("U*V - W = {}", left);
    println!("Expected: 0 (since T(r)=0)");

    // Print the polynomials in a human-readable format.
    println!("\nPolynomials (coefficients in ascending order of power):");
    println!("T(x) = {} + {} * x", t_coeffs[0], t_coeffs[1]);
    for i in 0..4 {
        println!("U_{}(x) = {}", i, u_coeffs[i]);
        println!("V_{}(x) = {}", i, v_coeffs[i]);
        println!("W_{}(x) = {}", i, w_coeffs[i]);
    }
}
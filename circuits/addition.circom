pragma circom 2.1.6;

/*
  AdditionDemo proves knowledge of two private values, a and b, such that:

      a + b = c

  The verifier learns only the public value c. The inputs a and b stay hidden
  because only c is listed as public in the main component.

  Cryptographic meaning:
  - Statement: "There exist private values a and b whose sum equals public c."
  - Witness: a, b, and all intermediate signal values.
  - Public input: c.
  - Constraint: the algebraic equation a + b === c over Circom's scalar field.

  Important field note:
  Circom arithmetic is finite-field arithmetic, not normal integer arithmetic.
  The equation is checked modulo the BN254 scalar field prime used by the
  proving backend. For small numbers like 3 + 4 = 7, this behaves exactly like
  ordinary integer arithmetic.
*/
template AdditionDemo() {
    /*
      Private witness inputs.

      Circom does not mark these with a `private` keyword. Inputs are private
      unless they are explicitly listed as public in the main component.
    */
    signal input a;
    signal input b;

    /*
      Public input.

      The `component main { public [c] } = AdditionDemo();` declaration below
      makes c visible to the verifier and Solidity verifier publicSignals.
    */
    signal input c;

    /*
      Intermediate signal.

      This makes the circuit easier to inspect and discuss. For such a small
      circuit, we could directly write `a + b === c`, but naming the sum helps
      when reading the witness and `.sym` files.
    */
    signal sum;

    /*
      `<==` assigns a witness value and adds the constraint that `sum` must
      equal `a + b`.

      Because this expression is linear, it does not require a multiplication
      gate. In R1CS terms, this is a linear equality constraint.
    */
    sum <== a + b;

    /*
      `===` adds an explicit equality constraint.

      This is the core proof condition: the prover cannot choose private
      values a and b unless their field sum equals the public value c.
    */
    sum === c;
}

/*
  Public signal layout:
  - c

  Example valid witness:
  - a = 3
  - b = 4
  - c = 7

  Example invalid witness:
  - a = 3
  - b = 4
  - c = 8

  Constraint estimate:
  - 1 linear equality for sum = a + b.
  - 1 linear equality for sum = c.
  - 0 multiplication constraints.

  Solidity verifier usage:
  After compiling and generating a Groth16 verifier, pass `[c]` as the public
  signals array. The proof hides a and b while proving that their sum equals c.
*/
component main { public [c] } = AdditionDemo();

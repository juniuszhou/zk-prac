# 处于Addition Circuit Demo

Circuit file: `circuits/addition.circom`

This is the first practical ZK exercise. It proves:

```text
I know private values a and b such that a + b = c.
```

The verifier sees `c`, but does not learn `a` or `b`.

## Circuit Logic

The circuit has three inputs:

- `a`: private witness input known only to the prover.
- `b`: private witness input known only to the prover.
- `c`: public input known to the verifier because it is listed in `component main { public [c] }`.

The circuit computes:

```text
sum = a + b
sum === c
```

If the prover cannot provide private values where `a + b = c`, proof generation or verification fails.

## Cryptographic Meaning

This circuit separates the ZK statement from the witness:

- Public statement: `c` is the claimed sum.
- Private witness: `a` and `b`.
- Relation: `a + b = c`.

The proof convinces the verifier that a valid witness exists without revealing the witness.

## Constraint Estimate

This circuit is intentionally cheap:

- `sum <== a + b`: one linear equality constraint.
- `sum === c`: one linear equality constraint.
- Multiplication constraints: `0`.

Addition is cheap in arithmetic circuits. Multiplication usually dominates constraint cost.

## Test Cases

Valid input:

```json
{
  "a": "3",
  "b": "4",
  "c": "7"
}
```

Invalid input:

```json
{
  "a": "3",
  "b": "4",
  "c": "8"
}
```

The valid input should generate and verify a proof. The invalid input should fail because the witness does not satisfy the constraint.

## Circom 产物说明

One `circom` compile produces two different kinds of output: a **circuit specification** for the proof system, and an **executable witness calculator** for the prover.

```bash
circom circuits/addition.circom --r1cs --wasm --sym -o build/addition
```

### Artifacts


| File                                                       | Kind    | Contents                                                                        | Used by                              |
| ---------------------------------------------------------- | ------- | ------------------------------------------------------------------------------- | ------------------------------------ |
| `addition.r1cs`                                            | Circuit | Field, wire count, public/private input layout, R1CS multiplication constraints | `snarkjs` setup, prove, `wtns check` |
| `addition_js/addition.wasm`                                | Program | WebAssembly that assigns every signal and runs `===` checks                     | Node (or browser) to compute witness |
| `addition_js/generate_witness.js`, `witness_calculator.js` | Glue    | Load wasm, feed `input.json`, write `witness.wtns`                              | `node generate_witness.js ...`       |
| `addition.sym`                                             | Symbols | Wire index to names like `main.a`, `main.c`                                     | Debugging, `snarkjs r1cs print`      |


Circom does **not** emit a Groth16 prover or verifier. Files such as `addition_final.zkey`, `verification_key.json`, and `AdditionVerifier.sol` come from later `snarkjs` commands plus a Powers of Tau file.

### Circuit vs program

**Circuit (`addition.r1cs`)** is a static math spec: which witness vectors are valid over the BN254 scalar field. Each R1CS constraint has the form (A \cdot w) \times (B \cdot w) = (C \cdot w). This file is not executed with `node`; `snarkjs` reads it for constraint checks and proving setup.

For this addition demo, constraints are only linear (`sum <== a + b`, `sum === c`), so `snarkjs r1cs info` reports `0` multiplication constraints. The relation `a + b = c` is enforced mainly inside the wasm witness calculator, not as R1CS multiplication gates.

**Program (`addition.wasm`)** is the witness calculator compiled from the same source. It exports functions such as `init`, `setInputSignal`, and `getWitness`. The JS wrapper hashes signal names from `input.json`, writes field elements into wasm memory, runs the circuit in topological order, and packs the result into `witness.wtns`.

circuit.wasm 是一个 WebAssembly 二进制模块，它是一个高效的 Witness 生成器（Witness Calculator）。

### **calculateWitness 高效的核心原因总结**

1. **预编译**：电路逻辑在 circom 编译阶段就翻译成了 Wasm 指令，不需要在运行时解释。
2. **极简执行路径**：几乎是“一条直线”执行计算，没有多余的抽象层。
3. **内存友好**：所有信号放在同一块连续内存中，CPU 缓存命中率极高。
4. **无运行时开销**：没有对象创建、没有垃圾回收、没有动态查找。
5. **并行潜力**：虽然单个 Witness 计算是单线程，但 Wasm 引擎（V8、SpiderMonkey）对这类计算做了很多底层优化。

出于安全性和可移植性的考虑，没有编译成机器代码。但是很多专业的 Prover在这样做了。使用C++来编译。

Same `input.json` always yields the same witness. Prover blinding randomness (`r`, `s` in Groth16) is **not** stored in the witness file; it is chosen later during `groth16 prove` and encoded into `proof.json` (`pi_a`, `pi_b`, `pi_c`).

Setup randomness (`tau`, `alpha`, `beta`, `delta`, and related values) lives in `.ptau` and `.zkey`, not in the witness.

### How artifacts are called

```text
addition.circom
    |
    +-- circom compile --> addition.r1cs, addition.wasm, addition.sym
    |
    +-- witness (wasm program)
    |       input.json --> node generate_witness.js addition.wasm ... --> witness.wtns
    |       optional: snarkjs wtns check addition.r1cs witness.wtns
    |
    +-- trusted setup (circuit + ptau)
    |       snarkjs groth16 setup --> addition_final.zkey, AdditionVerifier.sol
    |
    +-- prove (zkey + witness; no circom)
    |       snarkjs groth16 prove --> proof.json, public.json
    |
    +-- verify
            snarkjs groth16 verify (local)
            AdditionProofApp.verifyAddition --> AdditionVerifier.verifyProof (on-chain)
```

Equivalent witness command:

```bash
snarkjs wtns calculate \
  build/addition/addition_js/addition.wasm \
  build/addition/input.json \
  build/addition/witness.wtns
```

One-shot witness plus proof:

```bash
snarkjs groth16 fullprove \
  build/addition/input.json \
  build/addition/addition_js/addition.wasm \
  build/addition/addition_final.zkey \
  build/addition/proof.json \
  build/addition/public.json
```

On-chain verification only checks the proof and public inputs (for example `c = 7`). It does not run wasm and never sees `a` or `b`.

Optional compile flag `--c` emits C++ instead of wasm for faster native witness generation.

## Python End-to-End Demo

Run the full pipeline (compile, witness, setup, prove, snarkjs verify, Hardhat compile, on-chain verify):

```bash
npm install
python3 scripts/addition_circom_demo.py
```

Invalid input demo (`3 + 4 != 8` fails at witness generation):

```bash
python3 scripts/addition_circom_demo.py --invalid
```

Skip Hardhat steps:

```bash
python3 scripts/addition_circom_demo.py --skip-onchain
```

## Compile And Prove

Install tools if needed:

```bash
cargo install --git https://github.com/iden3/circom.git --tag v2.1.6
npm install -g snarkjs
```

Compile the circuit:

```bash
mkdir -p build/addition
circom circuits/addition.circom --r1cs --wasm --sym -o build/addition
```

Create the input file:

```bash
printf '{"a":"3","b":"4","c":"7"}\n' > build/addition/input.json
```

Generate the witness:

```bash
node build/addition/addition_js/generate_witness.js \
  build/addition/addition_js/addition.wasm \
  build/addition/input.json \
  build/addition/witness.wtns
```

Verify the witness:

```bash
snarkjs wtns check \
  build/addition/addition.r1cs \
  build/addition/witness.wtns
```

Inspect the constraint count:

```bash
snarkjs r1cs info build/addition/addition.r1cs
```

## Groth16 And Solidity Verifier

For Groth16, you need a Powers of Tau file. After setup, export the Solidity verifier:

```bash
snarkjs groth16 setup \
  build/addition/addition.r1cs \
  powersOfTau28_hez_final_08.ptau \
  build/addition/addition_0000.zkey

snarkjs zkey contribute \
  build/addition/addition_0000.zkey \
  build/addition/addition_final.zkey \
  --name="first contribution"

snarkjs zkey export solidityverifier \
  build/addition/addition_final.zkey \
  build/addition/AdditionVerifier.sol
```

Generate a proof and verification key:

```bash
snarkjs groth16 prove \
  build/addition/addition_final.zkey \
  build/addition/witness.wtns \
  build/addition/proof.json \
  build/addition/public.json

snarkjs zkey export verificationkey \
  build/addition/addition_final.zkey \
  build/addition/verification_key.json
```

Verify the proof locally:

```bash
snarkjs groth16 verify \
  build/addition/verification_key.json \
  build/addition/public.json \
  build/addition/proof.json
```

The Solidity verifier receives the public signal array:

```solidity
uint256[1] memory publicSignals = [uint256(7)];
```

The proof hides `a = 3` and `b = 4`, while proving their sum equals the public value `7`.
# zk-prac

## Learning Materials

- `docs/zero-knowledge-roadmap.md`: roadmap from ZK foundations to zkML.
- `docs/addition-circuit.md`: first Circom demo with commands, tests, constraints, and Solidity verifier notes.
- `docs/polkadot/hardhat-addition-deploy.md`: Hardhat deployment guide for Polkadot EVM and PVM targets.
- `circuits/addition.circom`: documented `a + b = c` circuit.
- `contracts/AdditionVerifier.sol`: Groth16 verifier generated from the addition circuit.
- `contracts/AdditionProofApp.sol`: Solidity wrapper used by tests and deployment scripts


## circom
compile
```bash
circom circuits/addition.circom --r1cs --wasm --sym -o build/addition
```

generate witness
```bash
snarkjs wtns calculate \
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




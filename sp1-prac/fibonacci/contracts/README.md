# Fibonacci SP1 On-Chain Verifier

Solidity app contract that verifies SP1 EVM proofs for the Fibonacci zkVM program.

## Architecture

```text
prover (evm.rs)  →  Groth16/PLONK proof + publicValues
                         ↓
Fibonacci.sol    →  ISP1Verifier.verifyProof(programVKey, publicValues, proof)
                         ↓
SP1Verifier*     →  pairing check (Succinct deployment or gateway)
```

- **`Fibonacci.sol`**: your app — checks the proof, decodes `(n, a, b)`.
- **`ISP1Verifier`**: Succinct’s generic SP1 verifier (not hand-written). Use a [deployed address](https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments) or `SP1VerifierGateway`.

## Setup

```sh
cd contracts
forge install   # already done if lib/ exists
forge build
forge test
```

## Generate real fixtures

From `script/` (needs Go + ~16GB RAM for Groth16/PLONK):

```sh
cargo run --release --bin evm -- --system groth16
cargo run --release --bin evm -- --system plonk
```

Writes `contracts/src/fixtures/groth16-fixture.json` and `plonk-fixture.json`.

## Program verification key

```sh
cd script
cargo run --release --bin vkey
```

## Deploy

```sh
VERIFIER=0x...          # SP1VerifierGateway on your chain
PROGRAM_VKEY=0x...       # from vkey binary

forge create src/Fibonacci.sol:Fibonacci \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --constructor-args $VERIFIER $PROGRAM_VKEY
```

## Call on-chain

```solidity
(uint32 n, uint32 a, uint32 b) = fibonacci.verifyFibonacciProof(publicValues, proofBytes);
```

`publicValues` and `proofBytes` come from the fixture / `evm.rs` output.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

### Development Workflow
- Compile contracts: `npm run compile`
- Type checking: `npm run typecheck`
- Run tests: `npm run test`
- Compile for PVM: `npm run compile:pvm`
- Run a single test: `npx hardhat test test/addition-proof.test.ts`

### Deployment Commands
- Local deployment: `npm run deploy:local`
- Polkadot EVM testnet: `npm run deploy:polkadot:evm`
- Polkadot PVM testnet: `npm run deploy:polkadot:pvm`

### Interaction Commands
- Verify proof via viem: `npm run call:verify:viem`
- Verify sample proof: `npm run verify-sample-proof`

### Circuit Operations
The circuits directory contains Circom files. To work with circuits:
1. Compile circuit: `circom addition.circom --r1cs --wasm -o build`
2. Generate witness: `node addition_js/generate_witness.js addition_js/addition.wasm input.json`
3. Generate proof: `snarkjs groth16 prove build/addition_0001.r1cs pot12_final.zkey witness.wtns proof.json public.json`
4. Verify proof: `snarkjs groth16 verify verification_key.json public.json proof.json`

## Code Architecture

### Circuit Layer (`circuits/`)
Contains Circom source files defining zk-SNARK circuits. The current `addition.circom` implements a simple constraint proving knowledge of private values a, b where a + b = c (public).

### Contracts Layer (`contracts/`)
Solidity smart contracts:
- `AdditionVerifier.sol`: Groth16 verifier contract generated from the circuit
- `AdditionProofApp.sol`: Application contract that uses the verifier to check proofs

### Scripts Layer (`scripts/`)
TypeScript utilities for:
- Proof generation and formatting (`proof-utils.ts`)
- Deployment scripts (`deploy-addition.ts`)
- Verification and interaction scripts (`call-addition-viem.ts`, `verify-sample-proof.ts`)

### Test Layer (`test/`)
Hardhat tests in TypeScript verifying the correctness of proof verification on-chain.
- `addition-proof.test.ts`: Tests the sample proof and validates rejection of incorrect public input.
- `fixtures/`: Contains sample proof and public inputs (`addition-proof.json`, `addition-public.json`) used by tests and verification scripts.

### Configuration
- `hardhat.config.ts`: Main Hardhat configuration for EVM networks
- `hardhat.pvm.config.ts`: Configuration for Polkadot PVM target
- `tsconfig.json`: TypeScript configuration
- `package.json`: Dependencies and npm scripts
- `.env.example`: Template for environment variables (PRIVATE_KEY, POLKADOT_TESTNET_RPC, ADDITION_APP_ADDRESS)

### Artifacts
Generated files appear in:
- `build/`: Circuit compilation artifacts (R1CS, WASM, zkey)
- `artifacts/`: Compiled contract artifacts
- `cache/`: Hardhat cache
- `ignition/`: Hardhat ignition modules
- `test/fixtures/`: Sample proofs and public inputs for testing

## Documentation and Learning Resources
- `docs/zero-knowledge-roadmap.md`: Roadmap from ZK foundations to zkML.
- `docs/addition-circuit.md`: Detailed explanation of the `a + b = c` circuit, constraints, and commands.
- `docs/polkadot/hardhat-addition-deploy.md`: Step‑by‑step guide for deploying to Polkadot EVM and PVM testnets.
- **Learning Goals** (from README.md):
    - @PROJECT_SPEC.md: Teach ZK basics from Interactive Proof to zk-SNARK, implement a simple "a+b=c" circuit in Circom.
    - @codebase: Help create an ONNX → EZKL flow for zkML (with steps: PyTorch MLP -> ONNX -> EZKL Halo2 circuit + proof -> Solidity verifier + Hardhat test, explaining quantization and lookup table optimizations).
- **Cursor Rules**:
    - `.cursor/rules/zk-fundamentals.md`: Instructions for the AI to act as a ZK expert, use latest best practices (2026), explain with cryptography principles + code implementation, and after generating a circuit provide: circuit logic explanation, estimated number of constraints, test cases, and Solidity verifier usage (if applicable).

## Typical Workflow
1. Define or modify a circuit in `circuits/`.
2. Compile the circuit to generate R1CS and WASM.
3. Use `snarkjs` to generate a proving key and verification key (already present as `pot12_final.zkey` and `verification_key.json` in this repo).
4. Create an input JSON with private values.
5. Generate a witness, then a proof.
6. Format the proof for Solidity using `scripts/proof-utils.ts`.
7. Deploy or interact with `AdditionProofApp.sol` to verify the proof on-chain.
8. Run tests to ensure correctness.

This repository demonstrates a full zk‑SNARK workflow: circuit definition → witness generation → proof creation → on‑chain verification using Solidity verifier contracts.
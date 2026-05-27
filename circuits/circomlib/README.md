# Vendored circomlib snippets

Files copied from [iden3/circomlib](https://github.com/iden3/circomlib) `master` (LGPL-3.0):

- `poseidon.circom`
- `poseidon_constants.circom`
- `mux1.circom`

`merkle.circom` is **not** in upstream circomlib; it implements `MerkleProof` for `circuits/vote.circom` using the Poseidon + `MultiMux1` inclusion pattern from community tutorials.

Compile from repo root:

```bash
circom circuits/vote.circom --r1cs --wasm -o build/vote
```

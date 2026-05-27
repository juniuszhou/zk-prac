pragma circom 2.1.6;

/*
  Merkle tree inclusion proof (Poseidon hash, binary tree).

  Note: iden3/circomlib does not ship a file named merkle.circom on master.
  This template follows the public MerkleTreeInclusionProof pattern used in
  community examples (Poseidon + MultiMux1), e.g. zkSNARK-playground
  examples/merkle-tree/tree.circom, adapted to the MerkleProof API used in
  circuits/vote.circom (leaf, pathElements, pathIndices -> root).

  Dependencies vendored from iden3/circomlib master:
    - poseidon.circom, poseidon_constants.circom, mux1.circom
*/

include "poseidon.circom";
include "mux1.circom";

template MerkleProof(levels) {
    signal input leaf;
    signal input pathElements[levels];
    signal input pathIndices[levels];
    signal output root;

    component poseidons[levels];
    component mux[levels];
    signal hashes[levels + 1];

    hashes[0] <== leaf;

    for (var i = 0; i < levels; i++) {
        pathIndices[i] * (1 - pathIndices[i]) === 0;

        poseidons[i] = Poseidon(2);
        mux[i] = MultiMux1(2);

        mux[i].c[0][0] <== hashes[i];
        mux[i].c[0][1] <== pathElements[i];
        mux[i].c[1][0] <== pathElements[i];
        mux[i].c[1][1] <== hashes[i];
        mux[i].s <== pathIndices[i];

        poseidons[i].inputs[0] <== mux[i].out[0];
        poseidons[i].inputs[1] <== mux[i].out[1];

        hashes[i + 1] <== poseidons[i].out;
    }

    root <== hashes[levels];
}

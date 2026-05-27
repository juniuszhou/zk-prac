pragma circom 2.1.6;

include "circomlib/merkle.circom";
include "circomlib/poseidon.circom";

template ZKVote(levels) {

    // ===== PUBLIC INPUTS =====
    signal input root;
    signal input nullifierHash;
    signal input vote;        // 0/1（也可以隐藏）

    // ===== PRIVATE INPUTS =====
    signal input identitySecret;
    signal input pathElements[levels];
    signal input pathIndices[levels];

    // ===== 1. membership proof =====
    component merkle = MerkleProof(levels);

    merkle.leaf <== identitySecret;
    for (var i = 0; i < levels; i++) {
        merkle.pathElements[i] <== pathElements[i];
        merkle.pathIndices[i] <== pathIndices[i];
    }

    // root must match
    root === merkle.root;

    // ===== 2. nullifier (prevent double voting) =====
    component hash = Poseidon(1);
    hash.inputs[0] <== identitySecret;

    nullifierHash === hash.out;

    // ===== 3. vote validity constraint =====
    // vote must be 0 or 1
    vote * (vote - 1) === 0;
}

component main { public [root, nullifierHash, vote] } = ZKVote(20);
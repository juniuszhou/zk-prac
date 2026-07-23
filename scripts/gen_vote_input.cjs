/**
 * Build input.json for circuits/vote.circom (ZKVote, depth 20).
 * Uses circomlib-compatible Poseidon + sparse Merkle path (leaf at index 0).
 *
 * Usage:
 *   node scripts/gen_vote_input.cjs build/vote/input.json
 *   node scripts/gen_vote_input.cjs build/vote/input.json --invalid
 */
const { writeFileSync } = require("node:fs");
const { buildPoseidon } = require("circomlibjs");

const LEVELS = 20;

function fieldToString(value) {
  return poseidon.F.toString(value);
}

function field(value) {
  return poseidon.F.e(value);
}

let poseidon;

function merkleRootFromLeaf(leaf) {
  const zeroHashes = [field(0)];
  for (let i = 1; i <= LEVELS; i++) {
    zeroHashes[i] = poseidon([zeroHashes[i - 1], zeroHashes[i - 1]]);
  }

  const pathElements = [];
  const pathIndices = [];
  let current = leaf;

  for (let i = 0; i < LEVELS; i++) {
    pathIndices.push(0);
    pathElements.push(zeroHashes[i]);
    current = poseidon([current, zeroHashes[i]]);
  }

  return { root: current, pathElements, pathIndices };
}

async function main() {
  const invalid = process.argv.includes("--invalid");
  const outPath =
    process.argv.find((arg) => arg.endsWith(".json")) ||
    "build/vote/input.json";

  poseidon = await buildPoseidon();

  const identitySecret = field("12345");
  const vote = "1";
  const nullifierHash = poseidon([identitySecret]);
  const { root, pathElements, pathIndices } = merkleRootFromLeaf(identitySecret);
  const badRoot = poseidon.F.add(root, field(1));

  const input = {
    root: fieldToString(invalid ? badRoot : root),
    nullifierHash: fieldToString(nullifierHash),
    vote,
    identitySecret: fieldToString(identitySecret),
    pathElements: pathElements.map(fieldToString),
    pathIndices: pathIndices.map((index) => index.toString())
  };

  writeFileSync(outPath, `${JSON.stringify(input, null, 2)}\n`, "utf8");
  console.log(`Wrote ${outPath}`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});

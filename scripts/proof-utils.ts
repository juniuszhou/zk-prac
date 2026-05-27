import { readFileSync } from "node:fs";

export type SnarkProof = {
  pi_a: [string, string, string];
  pi_b: [[string, string], [string, string], [string, string]];
  pi_c: [string, string, string];
};

export type SolidityProof = {
  pA: [string, string];
  pB: [[string, string], [string, string]];
  pC: [string, string];
  publicSignals: string[];
};

export function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

export function formatProofForSolidity(
  proof: SnarkProof,
  publicSignals: string[]
): SolidityProof {
  return {
    pA: [proof.pi_a[0], proof.pi_a[1]],
    pB: [
      [proof.pi_b[0][1], proof.pi_b[0][0]],
      [proof.pi_b[1][1], proof.pi_b[1][0]]
    ],
    pC: [proof.pi_c[0], proof.pi_c[1]],
    publicSignals
  };
}

export function readAdditionProof(
  proofPath = "test/fixtures/addition-proof.json",
  publicPath = "test/fixtures/addition-public.json"
): SolidityProof {
  return formatProofForSolidity(
    readJson<SnarkProof>(proofPath),
    readJson<string[]>(publicPath)
  );
}

/**
 * Verify proof.json / public.json produced by scripts/addition_circom_demo.py
 * on a local Hardhat network (no RPC required).
 */
import { ethers } from "hardhat";

import { formatProofForSolidity, readJson, type SnarkProof } from "./proof-utils";

const BUILD_DIR = "build/addition";

async function main() {
  const proof = readJson<SnarkProof>(`${BUILD_DIR}/proof.json`);
  const publicSignals = readJson<string[]>(`${BUILD_DIR}/public.json`);
  const { pA, pB, pC, publicSignals: pub } = formatProofForSolidity(proof, publicSignals);

  console.log("publicSignals:", pub);

  const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier");
  const verifier = await Groth16Verifier.deploy();
  await verifier.waitForDeployment();

  const AdditionProofApp = await ethers.getContractFactory("AdditionProofApp");
  const app = await AdditionProofApp.deploy(await verifier.getAddress());
  await app.waitForDeployment();

  const valid = await app.verifyAdditionView(pub[0], pA, pB, pC);
  console.log("On-chain verifyAdditionView:", valid);

  if (!valid) {
    throw new Error("Fresh proof did not verify on Hardhat");
  }

  const wrongSum = await app.verifyAdditionView(8, pA, pB, pC);
  console.log("Wrong public sum (8) should be false:", wrongSum);
  if (wrongSum) {
    throw new Error("Expected wrong public input to fail verification");
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

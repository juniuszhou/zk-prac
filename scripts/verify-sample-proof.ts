import { ethers } from "hardhat";

import { readAdditionProof } from "./proof-utils";

async function main() {
  const appAddress = process.env.ADDITION_APP_ADDRESS;

  if (!appAddress) {
    throw new Error("Set ADDITION_APP_ADDRESS to the deployed AdditionProofApp address.");
  }

  const { pA, pB, pC, publicSignals } = readAdditionProof();
  const app = await ethers.getContractAt("AdditionProofApp", appAddress);
  const publicSum = publicSignals[0];

  const isValid = await app.verifyAdditionView(publicSum, pA, pB, pC);
  console.log(`Proof for public sum ${publicSum}:`, isValid ? "valid" : "invalid");
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});

import { ethers } from "hardhat";

import { readAdditionProof } from "../scripts/proof-utils";

async function deployAdditionProofApp() {
  const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier");
  const verifier = await Groth16Verifier.deploy();

  const AdditionProofApp = await ethers.getContractFactory("AdditionProofApp");
  return AdditionProofApp.deploy(await verifier.getAddress());
}

describe("AdditionProofApp", function () {
  it("verifies the Circom addition proof", async function () {
    const app = await deployAdditionProofApp();
    const { pA, pB, pC, publicSignals } = readAdditionProof();

    const isValid = await app.verifyAdditionView(publicSignals[0], pA, pB, pC);

    if (!isValid) {
      throw new Error("Expected sample proof to verify");
    }
  });

  it("rejects the same proof with the wrong public sum", async function () {
    const app = await deployAdditionProofApp();
    const { pA, pB, pC } = readAdditionProof();
    const isValid = await app.verifyAdditionView(8, pA, pB, pC);

    if (isValid) {
      throw new Error("Expected proof with wrong public input to fail");
    }
  });
});

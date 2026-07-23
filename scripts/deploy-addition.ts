import { ethers, network } from "hardhat";

import { requireDeployer } from "./lib/deploy-utils";

async function main() {
  const deployer = await requireDeployer();

  const Groth16Verifier = await ethers.getContractFactory(
    "Groth16Verifier",
    deployer
  );
  const verifier = await Groth16Verifier.deploy();
  await verifier.waitForDeployment();
  const verifierAddress = await verifier.getAddress();

  const AdditionProofApp = await ethers.getContractFactory(
    "AdditionProofApp",
    deployer
  );
  const app = await AdditionProofApp.deploy(verifierAddress);
  await app.waitForDeployment();
  const appAddress = await app.getAddress();

  console.log("Network:", network.name);
  console.log("Groth16Verifier:", verifierAddress);
  console.log("AdditionProofApp:", appAddress);
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});

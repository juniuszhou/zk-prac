import { ethers, network } from "hardhat";

async function main() {
  const Groth16Verifier = await ethers.getContractFactory("Groth16Verifier");
  const verifier = await Groth16Verifier.deploy();
  await verifier.waitForDeployment();
  const verifierAddress = await verifier.getAddress();

  const AdditionProofApp = await ethers.getContractFactory("AdditionProofApp");
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

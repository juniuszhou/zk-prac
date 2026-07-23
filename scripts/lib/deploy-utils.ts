import { ethers, network } from "hardhat";
import type { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";

export async function requireDeployer(): Promise<HardhatEthersSigner> {
  const signers = await ethers.getSigners();
  if (signers.length === 0) {
    throw new Error(
      [
        "No deployer account for this network.",
        "Copy .env.example to .env and set PRIVATE_KEY (0x + 64 hex characters).",
        "Then fund that address on Polkadot Hub TestNet before deploying."
      ].join(" ")
    );
  }

  const deployer = signers[0];
  const balance = await ethers.provider.getBalance(deployer.address);

  console.log("Deployer:", deployer.address);
  console.log("Balance:", ethers.formatEther(balance), "PAS");

  if (balance === 0n && network.name !== "hardhat") {
    throw new Error(
      `Deployer ${deployer.address} has zero balance on ${network.name}.`
    );
  }

  return deployer;
}

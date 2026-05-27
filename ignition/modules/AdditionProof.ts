import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

export default buildModule("AdditionProofModule", (m) => {
  const verifier = m.contract("Groth16Verifier");
  const app = m.contract("AdditionProofApp", [verifier]);

  return { verifier, app };
});

import "dotenv/config";

import { createPublicClient, defineChain, http, parseAbi } from "viem";

import { readAdditionProof } from "./proof-utils";

const additionProofAppAbi = parseAbi([
  "function verifyAdditionView(uint256 publicSum, uint256[2] pA, uint256[2][2] pB, uint256[2] pC) view returns (bool)"
]);

const polkadotTestnet = defineChain({
  id: 420420417,
  name: "Polkadot Hub TestNet",
  nativeCurrency: {
    decimals: 18,
    name: "PAS",
    symbol: "PAS"
  },
  rpcUrls: {
    default: {
      http: [process.env.POLKADOT_TESTNET_RPC || "https://services.polkadothub-rpc.com/testnet"]
    }
  }
});

function toBigIntPair(values: [string, string]): readonly [bigint, bigint] {
  return [BigInt(values[0]), BigInt(values[1])] as const;
}

function readProofForSolidity() {
  const { pA, pB, pC, publicSignals } = readAdditionProof();

  return {
    publicSum: BigInt(publicSignals[0]),
    pA: toBigIntPair(pA),
    pB: [
      toBigIntPair(pB[0]),
      toBigIntPair(pB[1])
    ],
    pC: toBigIntPair(pC)
  } as const;
}

async function main() {
  const appAddress = process.env.ADDITION_APP_ADDRESS;

  if (!appAddress) {
    throw new Error("Set ADDITION_APP_ADDRESS to the deployed AdditionProofApp address.");
  }

  const rpcUrl = process.env.POLKADOT_TESTNET_RPC || polkadotTestnet.rpcUrls.default.http[0];
  const client = createPublicClient({
    chain: polkadotTestnet,
    transport: http(rpcUrl)
  });
  const { publicSum, pA, pB, pC } = readProofForSolidity();

  const valid = await client.readContract({
    address: appAddress as `0x${string}`,
    abi: additionProofAppAbi,
    functionName: "verifyAdditionView",
    args: [publicSum, pA, pB, pC]
  });

  console.log(`Contract: ${appAddress}`);
  console.log(`RPC: ${rpcUrl}`);
  console.log(`Public sum: ${publicSum}`);
  console.log(`Proof valid: ${valid}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

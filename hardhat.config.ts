import "dotenv/config";
import "@nomicfoundation/hardhat-ethers";
import "@nomicfoundation/hardhat-ignition";
import "@nomicfoundation/hardhat-verify";
import "@parity/hardhat-polkadot";

import type { HardhatUserConfig } from "hardhat/config";

const PRIVATE_KEY = process.env.PRIVATE_KEY;
const POLKADOT_TESTNET_RPC =
  process.env.POLKADOT_TESTNET_RPC || "https://services.polkadothub-rpc.com/testnet";
const POLKADOT_TESTNET_CHAIN_ID = 420420417;

const accounts = PRIVATE_KEY ? [PRIVATE_KEY] : [];

const config: HardhatUserConfig = {
  solidity: {
    version: "0.8.28",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200
      }
    }
  },
  resolc: {
    compilerSource: "npm"
  },
  networks: {
    hardhat: {},
    polkadotTestnet: {
      polkadot: {
        target: "evm"
      },
      url: POLKADOT_TESTNET_RPC,
      chainId: POLKADOT_TESTNET_CHAIN_ID,
      accounts
    },
    polkadotPvmTestnet: {
      polkadot: {
        target: "pvm"
      },
      url: POLKADOT_TESTNET_RPC,
      chainId: POLKADOT_TESTNET_CHAIN_ID,
      accounts
    }
  },
  etherscan: {
    apiKey: {
      polkadotTestnet: "no-api-key-needed"
    },
    customChains: [
      {
        network: "polkadotTestnet",
        chainId: POLKADOT_TESTNET_CHAIN_ID,
        urls: {
          apiURL: "https://blockscout-testnet.polkadot.io/api",
          browserURL: "https://blockscout-testnet.polkadot.io/"
        }
      }
    ]
  },
  ignition: {
    requiredConfirmations: 1
  }
};

export default config;

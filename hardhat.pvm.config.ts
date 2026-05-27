import baseConfig from "./hardhat.config";
import type { HardhatUserConfig } from "hardhat/config";

const config: HardhatUserConfig = {
  ...baseConfig,
  paths: {
    ...baseConfig.paths,
    artifacts: "artifacts-pvm",
    cache: "cache-pvm"
  }
};

export default config;

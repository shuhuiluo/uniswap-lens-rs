import "@nomicfoundation/hardhat-foundry";
import "@nomiclabs/hardhat-ethers";
import "@typechain/hardhat";
import { HardhatUserConfig } from "hardhat/config";
import { SolidityUserConfig } from "hardhat/types/config";

/**
 * @type import('hardhat/config').HardhatUserConfig
 */
const config: HardhatUserConfig = {
  solidity: {
    version: "0.8.28",
    settings: {
      optimizer: {
        enabled: true,
        runs: 4294967295,
      },
      viaIR: false,
      evmVersion: "paris",
      metadata: {
        appendCBOR: false,
        bytecodeHash: "none",
      },
    },
  } as SolidityUserConfig,
  typechain: {
    outDir: "typechain",
    target: "ethers-v5",
  },
  networks: {
    hardhat: {
      chainId: 1,
      loggingEnabled: false,
      accounts: [],
    },
  },
  paths: {
    tests: "./test/hardhat",
  },
};

export default config;

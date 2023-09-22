// This script verifies the PublicVoting.sol contract on etherscan.io.
// Given the public keys of the ZK node, the script converts the public keys to Ethereum addresses
// for the contract constructor.


const CONTRACT_ADDRESS = process.env.ETH_CONTRACT_ADDRESS;
const PBC_CONTRACT_ADDRESS = process.env.PBC_CONTRACT_ADDRESS;
const ZK_ENGINE_PUB_KEY_0 = process.env.ZK_ENGINE_PUB_KEY_0;
const ZK_ENGINE_PUB_KEY_1 = process.env.ZK_ENGINE_PUB_KEY_1;
const ZK_ENGINE_PUB_KEY_2 = process.env.ZK_ENGINE_PUB_KEY_2;
const ZK_ENGINE_PUB_KEY_3 = process.env.ZK_ENGINE_PUB_KEY_3;

const hre = require("hardhat");
const computeEthereumAddress = require("public-voting/scripts/pbc");

async function main() {
  const nodeAddresses = [
    computeEthereumAddress(ZK_ENGINE_PUB_KEY_0, "base64"),
    computeEthereumAddress(ZK_ENGINE_PUB_KEY_1, "base64"),
    computeEthereumAddress(ZK_ENGINE_PUB_KEY_2, "base64"),
    computeEthereumAddress(ZK_ENGINE_PUB_KEY_3, "base64"),
  ];

  await hre.run("verify:verify", {
    address: "0x" + CONTRACT_ADDRESS,
    constructorArguments: ["0x" + PBC_CONTRACT_ADDRESS, nodeAddresses],
  });
}

main()
.then(() => process.exit(0))
.catch((error) => {
      console.error(error);
      process.exit(1);
    },
);
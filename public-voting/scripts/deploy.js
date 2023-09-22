// This script deploys the PublicVoting.sol contract to Ethereum.
// Given the public keys of the ZK node, the script converts the public keys to Ethereum addresses
// for the contract constructor.

const PBC_CONTRACT_ADDRESS = process.env.PBC_CONTRACT_ADDRESS;
const ZK_ENGINE_PUB_KEY_0 = process.env.ZK_ENGINE_PUB_KEY_0;
const ZK_ENGINE_PUB_KEY_1 = process.env.ZK_ENGINE_PUB_KEY_1;
const ZK_ENGINE_PUB_KEY_2 = process.env.ZK_ENGINE_PUB_KEY_2;
const ZK_ENGINE_PUB_KEY_3 = process.env.ZK_ENGINE_PUB_KEY_3;

const computeEthereumAddress = require("public-voting/scripts/pbc");
const hre = require("hardhat");
const ethers = hre.ethers;

async function main() {
  const [deployer] = await ethers.getSigners();

  console.log("%c \n Deploying contracts with the account:", "color:", deployer.address);

  console.log("%c \n Account balance:", "color:", (await deployer.getBalance()).toString());

  const nodeAddresses = [
    computeEthereumAddress(ZK_ENGINE_PUB_KEY_0, "base64"),
    computeEthereumAddress(ZK_ENGINE_PUB_KEY_1, "base64"),
    computeEthereumAddress(ZK_ENGINE_PUB_KEY_2, "base64"),
    computeEthereumAddress(ZK_ENGINE_PUB_KEY_3, "base64"),
  ];

  const PublicVoting = await ethers.getContractFactory("PublicVoting");

  const public_voting = await PublicVoting.deploy(
      "0x" + PBC_CONTRACT_ADDRESS,
      nodeAddresses);
  console.log("Contract deployed to address: " + public_voting.address);
}

main()
.then(() => process.exit(0))
.catch((error) => {
      console.error(error);
      process.exit(1);
    },
);
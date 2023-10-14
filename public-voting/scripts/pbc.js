const EC = require("elliptic").ec;
const hre = require("hardhat");
const ethers = hre.ethers;

const curve = new EC("secp256k1");

function testKey(key) {
  console.log(`Deriving from:    ${key}`);

  let buffer = Buffer.from(key, "Base64");
  let publicKey = ethers.utils.hexlify(buffer);

  let uncompressedEcPoint = curve.keyFromPublic(buffer).getPublic().encode("hex", false);
  console.log(`EC point:         ${uncompressedEcPoint.substring(2)}`)

  console.log(`Ethereum Address: ${ethers.utils.computeAddress(publicKey).substring(2)}`)
}

module.exports = {testKey};

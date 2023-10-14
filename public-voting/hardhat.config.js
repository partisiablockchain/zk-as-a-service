/** @type import('hardhat/config').HardhatUserConfig */

require("dotenv").config();
require("@nomiclabs/hardhat-ethers");
require("@nomiclabs/hardhat-etherscan");
require('solidity-coverage');

const {GOERLI_API_URL, GOERLI_PRIVATE_KEY, ETHERSCAN_API_KEY} = process.env;

module.exports = {
  solidity: "0.8.18",
  // defaultNetwork: "goerli",
  networks: {
    hardhat: {},
    // goerli: {
    //   url: GOERLI_API_URL,
    //   accounts: [`0x${GOERLI_PRIVATE_KEY}`],
    // },
  },
  etherscan: {
    // Your API key for Etherscan
    // Obtain one at https://etherscan.io/
    apiKey: ETHERSCAN_API_KEY,
  },
};

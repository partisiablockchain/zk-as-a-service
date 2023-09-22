# ZK as a service

This repository contains example code for using ZK-as-a-service as a second layer for an existing
EVM app that can display the result of a secret vote.

An overview of how PBC can be used as a second layer solution can be found [here](https://partisiablockchain.gitlab.io/documentation/PBC-as-a-second-layer/pbc-as-second-layer.html).

## Try it out

If you wish to try out a live demo of this example code, each contract has been deployed to the 
[PBC testnet](https://partisiablockchain.gitlab.io/documentation/testnet.html) and the [Ethereum 
Goerli testnet](https://goerli.etherscan.io/)

[This page](https://partisiablockchain.gitlab.io/documentation/PBC-as-a-second-layer/pbc-as-a-second-layer-live-example-ethereum.html)
describes how to run the example.

## How to deploy

Having run the example you may wish to try to deploy these contracts yourself. 

If you have made any modifications to the code, make sure that you understand how data is moved from
PBC to Ethereum to ensure that it can still happen in a secure manner.

If you wish to know more, you can read details on how it works [here](https://partisiablockchain.gitlab.io/documentation/PBC-as-a-second-layer/pbc-as-a-second-layer-how-to-create-your-own-solution.html).

More information on how to deploy the solution can be found [here](https://partisiablockchain.gitlab.io/documentation/PBC-as-a-second-layer/pbc-as-a-second-layer-how-to-deploy.html).

### Requirements

In order to be able to compile and deploy the contracts, the following tools are required.

* [Rust version >= 1.64.0](https://rustup.rs)
* [partisia-contract version >= 1.25.0](https://crates.io/crates/cargo-partisia-contract)
* [Node.js version = 16.15.0](https://nodejs.org/en)

### Build and deploy PBC private smart contract

From working directory `./`.

1. Build the PBC contract using the command 
    ```shell
    cargo partisia-contract build --release
    ```
    The outputs are located at `./target/wasm32-unknown-unknown/release/private_voting.{abi, zkwa}`.
2. Deploy PBC contract on [PBC testnet](https://testnet.partisiablockchain.com/wallet/upload_wasm). 
3. Copy and save the address of the deployed contract, and the public keys of the chosen ZK 
   computation nodes.

### Deploy solidity contract

From working directory `./public-voting`.

1. Create a file called `.env` in the `./public-voting`.
2. Fill in the values of the `.env` file.
   ```text
   API_URL = "<ENDPOINT_TO_GOERLI_TESTNET>"
   PRIVATE_KEY = "<GOERLI_TESTNET_PRIVATE_KEY>"
   PBC_CONTRACT_ADDRESS = "<ADDRESS_OF_NEWLY_DEPLOYED_PBC_CONTRACT>"
   ZK_ENGINE_PUB_KEY_0 = "<1ST_BASE64_ENCODED_PUBLIC_KEY_FOUND_IN_ZK_STATE>"
   ZK_ENGINE_PUB_KEY_1 = "<2ND_BASE64_ENCODED_PUBLIC_KEY_FOUND_IN_ZK_STATE>"
   ZK_ENGINE_PUB_KEY_2 = "<3RD_BASE64_ENCODED_PUBLIC_KEY_FOUND_IN_ZK_STATE>"
   ZK_ENGINE_PUB_KEY_3 = "<4TH_BASE64_ENCODED_PUBLIC_KEY_FOUND_IN_ZK_STATE>"
   ```
3. In the `hardhat.config.js` file, uncomment the lines
   ```text
   defaultNetwork: "goerli",
   ```
   and 
   ```text
   goerli: {
     url: API_URL,
     accounts: [`0x${PRIVATE_KEY}`],
   },
   ```
   to enable the Goerli network when running hardhat scripts.
4. Run the command
   ```shell
   npx hardhat run scripts/deploy.js --network goerli
   ```
   
## Helper scripts

The `./public-voting/scripts` directory contains several scripts to help with deploying the solidity
contract to Ethereum.

### deploy.js

This script help with deploying the PublicVoting.sol contract, using the hardhat runtime.

To work correctly, it expects that the environment variables `API_URL`, `PRIVATE_KEY`, 
`PBC_CONTRACT_ADDRESS`, `ZK_ENGINE_PUB_KEY_0`, `ZK_ENGINE_PUB_KEY_1`, `ZK_ENGINE_PUB_KEY_2`, and 
`ZK_ENGINE_PUB_KEY_3` are set. See above for details.

Given the environment variables, the script formats them to form valid constructor arguments before
deploying.

The deploy.js script can be run with the command:

```shell
npx hardhat run scripts/deploy.js --network goerli
```

### verify.js

This script can be used to verify a deployed contract on the 
[etherscan.io](https://etherscan.io/) block explorer.

Verifying contract provides etherscan with an ABI for the contract, making it possible to display
the contract state and to interact with the contract via the etherscan UI.

For the script to work it expects the same environment variables as the deploy.js script.
Additionally, the variables `ETH_CONTRACT_ADDRESS` and `ETHERSCAN_API_KEY` must also be available.

The script formats the input from the deploy.js script, which is needed when verifying.

The script can be run with the command

```shell
npx hardhat run scripts/verify.js --network goerli
```

### pbc.js

This script can convert a public key, provided by a computation node on PBC, to an Ethereum address.

It is used by the deploy.js and verify.js scripts to format the constructor arguments.
It can also be used from the command line, to manually verify that the addresses on Ethereum 
are correctly derived from a public key from PBC.

Given a public key from PBC `A/J83e6pRe9ARxCJNrUxu2iVfh3HTKk4CEYyZFxWn4NG`, the script can be run
with the command

```shell
npx run-func scripts/pbc.js testKey "A9SqNrfygSuXOLNsdy4Gx8d0kSV5S/ET7GCnTVz90FQ7"
```

```text
Deriving from:    A9SqNrfygSuXOLNsdy4Gx8d0kSV5S/ET7GCnTVz90FQ7
EC point:         d4aa36b7f2812b9738b36c772e06c7c7749125794bf113ec60a74d5cfdd0543bab6ac00abfa468e83e14ea45541e7d3c479c9af4b8d7a83ce8300a88fc20043d
Ethereum Address: b6e8E2BD838518F6Ac146fa6f7271c023723C0d0

```



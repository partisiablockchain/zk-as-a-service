const {loadFixture} = require("@nomicfoundation/hardhat-network-helpers");
const {expect} = require("chai");
const {revertedWith} = require("@nomicfoundation/hardhat-chai-matchers");

describe("PublicVoting contract", function() {
  const domainSeparator = "ZK_REAL_ATTESTATION";
  const pbcContractAddress = "0x030000000000000000000000000000000000424242";
  const pbcAccount1 = "0x000000000000000000000000000000000000002112";
  const pbcAccount2 = "0x000000000000000000000000000000000000001221";

  async function deployPublicVotingFixture() {
    let nodes = [];
    let nodeAddresses = [];
    for (let i = 0; i < 4; i++) {
      let testWallet = ethers.Wallet.createRandom();
      nodes[i] = testWallet;
      nodeAddresses[i] = testWallet.address;
    }
    const PublicVoting = await ethers.getContractFactory("PublicVoting");

    const [owner, addr1, addr2] = await ethers.getSigners();
    const hardhatVoting = await PublicVoting.deploy(pbcContractAddress, nodeAddresses);
    return {
      PublicVoting,
      hardhatVoting,
      pbcContractAddress,
      pbcAccount1,
      pbcAccount2,
      nodes,
      nodeAddresses,
      owner,
      addr1,
      addr2,
    };
  }

  describe("Deployment", function() {

    it("Set right pbc contract address", async function() {
      const {pbcContractAddress, hardhatVoting} = await loadFixture(deployPublicVotingFixture);
      expect(await hardhatVoting.privateVotingPbcAddress()).to.equal(pbcContractAddress);
    });

    it("Set node addresses in the right order", async function() {
      const {nodes, hardhatVoting} = await loadFixture(deployPublicVotingFixture);
      for (let i = 0; i < 4; i++) {
        expect(await hardhatVoting.computationNodes(i)).to.equal(nodes[i].address);
      }
    });

    it("Deployment fails if not enough node addresses supplied", async function() {
      const {pbcContractAddress, nodeAddresses, PublicVoting} = await loadFixture(
          deployPublicVotingFixture);
      await expect(PublicVoting.deploy(pbcContractAddress, nodeAddresses.slice(1))).to.be
      .revertedWith("Invalid computation node count");
    });

    it("Deployment fails if too many public keys supplied", async function() {
      const {pbcContractAddress, PublicVoting} = await loadFixture(
          deployPublicVotingFixture);
      let addresses = [];
      for (let i = 0; i < 5; i++) {
        const randomWallet = ethers.Wallet.createRandom();
        addresses[i] = randomWallet.address;
      }
      await expect(PublicVoting.deploy(pbcContractAddress, addresses)).to.be
      .revertedWith("Invalid computation node count");
    });
  });

  describe("Publishing result", function() {
    it("Publish result of a vote", async function() {
      const {nodes, hardhatVoting} = await loadFixture(deployPublicVotingFixture);
      const voteId = 1;
      const votesFor = 101;
      const votesAgainst = 50;
      const proof = generateProof(nodes, pbcContractAddress, voteId, votesFor, votesAgainst);

      await expect(() => {
        hardhatVoting.results(0);
      }).to.throw;

      await expect(hardhatVoting.publishResult(voteId, votesFor, votesAgainst, proof))
      .to.emit(hardhatVoting, "VotingCompleted")
      .withArgs([voteId, votesFor, votesAgainst]);

      expect(await hardhatVoting.results(0)).to.have.members(
          [voteId, votesFor, votesAgainst]);
    });

    it("Result of a vote cannot be tampered with", async function() {
      const {nodes, hardhatVoting} = await loadFixture(deployPublicVotingFixture);
      const voteId = 1;
      const votesFor = 101;
      const votesAgainst = 50;
      const proof = generateProof(nodes, pbcContractAddress, voteId, votesFor, votesAgainst);

      await expect(
          hardhatVoting.publishResult(voteId + 1, votesFor, votesAgainst, proof))
      .to.be.revertedWith("Could not verify signature");

      await expect(
          hardhatVoting.publishResult(voteId, votesFor + 1, votesAgainst, proof))
      .to.be.revertedWith("Could not verify signature");

      await expect(
          hardhatVoting.publishResult(voteId, votesFor, votesAgainst + 1, proof))
      .to.be.revertedWith("Could not verify signature");
    });

    it("Result cannot be verified with less than 4 signatures", async function() {
      const {nodes, hardhatVoting} = await loadFixture(deployPublicVotingFixture);
      const voteId = 1;
      const votesFor = 101;
      const votesAgainst = 50;
      const proof = generateProof(nodes, pbcContractAddress, voteId, votesFor, votesAgainst);

      await expect(
          hardhatVoting.publishResult(voteId + 1, votesFor, votesAgainst, proof.slice(0, 2)))
      .to.be.revertedWith("Not enough signatures");
    });
  });

  /// Utility and helper methods
  function generateProof(nodes, contract, voteId, votesFor, votesAgainst) {
    const digest = attestationDigest(contract, voteId, votesFor, votesAgainst);
    let proof = [];
    for (let i = 0; i < 4; i++) {
      proof[i] = sign(digest, nodes[i]._signingKey());
    }
    return proof;
  }

  function attestationDigest(contract, voteId, votesFor, votesAgainst) {
    return ethers.utils.soliditySha256(
        ["string", "bytes21", "uint32", "uint32", "uint32"],
        [domainSeparator, contract, voteId, votesFor, votesAgainst]);
  }

  function sign(digest, signingKey) {
    const signature = signingKey.signDigest(digest);
    return ethers.utils.joinSignature(signature);
  }
});
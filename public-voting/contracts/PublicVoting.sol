// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/// @title Public Voting using PBC for ZK-as-a-service
/// @author Partisia Blockchain
/// @notice This contract is used for publishing the result of a secret vote.
contract PublicVoting {

    /// @notice Representation of a result for a vote.
    struct VoteResult {
        // Identifier for the vote
        uint32 voteId;
        // Number of votes in favor
        uint32 votesFor;
        // Number of votes against
        uint32 votesAgainst;
    }

    /// @notice Event for when a the result of a vote has been validated
    /// @param result the result of the vote
    event VotingCompleted (VoteResult result);

    /// @notice Event for notifying that the PBC connection of the contract has been reset.
    /// @param newPbcAddress for the pbc contract
    /// @param newComputationNodes addresses derived from nodes public keys
    event PbcConnectionReset (bytes21 newPbcAddress, address[] newComputationNodes);

    // PBC address of the private contract that handles the secret voting input and
    // computing the result. Note that this contract must exist on the Partisia Blockchain.
    // Field must be public such that anyone can validate the state of this contract corresponds to
    // the state on PBC.
    bytes21 public privateVotingPbcAddress;
    // (ETH) addresses derived from the public keys of the computation nodes responsible for
    // the actual ZK computation (i.e. receiving secret inputs and counting votes). The public keys
    // can be read from the state of the private contract on PBC.
    // Field must be public such that anyone can validate the state of this contract corresponds to
    // the state on PBC.
    address[] public computationNodes;

    // List of voting results that has been validated.
    VoteResult[] public results;

    /// @notice Constructor for the public contract
    /// @param _pbcContractAddress address of the private voting contract
    /// @param _computationNodes addresses derived from the computation nodes' public keys
    constructor(bytes21 _pbcContractAddress, address[] memory _computationNodes) {
        privateVotingPbcAddress = _pbcContractAddress;
        require(_computationNodes.length == 4, "Invalid computation node count");
        computationNodes = _computationNodes;
    }

    /// @notice Declare and verify the result of the vote
    /// @param _voteId the identifier for the poll to check result for
    /// @param _votesFor number of votes in favor
    /// @param _votesAgainst number of votes against
    /// @param _proofOfResult a collection of signatures attesting the voting result
    /// @dev All the data needed to publish the result of a vote can be found on the public state
    ///      of the PBC contract. The result contains the vote id, the for and against count and
    ///      the proof of the result, provided by the computation nodes.
    function publishResult(
        uint32 _voteId,
        uint32 _votesFor,
        uint32 _votesAgainst,
        bytes[] calldata _proofOfResult) external {

        // Verify that we have signatures from all of the computation nodes.
        require(_proofOfResult.length == 4, "Not enough signatures");
        // Compute the SHA-256 hash value (also called digest) of the data that was signed by the
        // computation nodes. The nodes have not signed the raw data, but rather the digest of the
        // data, so we need to compute it here to verify the signatures against it.
        bytes32 digest = computeDigest(_voteId, _votesFor, _votesAgainst);

        // For each of the 4 signatures:
        for (uint32 node = 0; node < 4; node++) {
            // The the signature from the input array.
            bytes calldata signature = _proofOfResult[node];
            // Verify that the address recovered from the signature matches one of the computation
            // nodes that we trust from the initialization of the contract.
            require(computationNodes[node] == ECDSA.recover(digest, signature),
                "Could not verify signature");
        }

        // All signatures were verified so we can publish the result of the vote.
        VoteResult memory result = VoteResult(_voteId, _votesFor, _votesAgainst);
        emit VotingCompleted(result);
        results.push(result);
    }

    /// @notice Compute the digest of the attested data
    /// @dev The digest of a voting result is the 32 bytes of data that has been signed by the
    ///      trusted computation nodes. The digest can be thought of as a fingerprint of the data.
    ///      We need to compute the digest here from the voting result, since we cannot extract the
    ///      result data from the digest, and the digest is needed to verify the signatures from the
    ///      proof.
    function computeDigest(
        uint32 _voteId,
        uint32 _votesFor,
        uint32 _votesAgainst) private view returns (bytes32) {
        // The digest of the attested data follows the format:
        // sha256(attestation_domain_separator || pbc_contract_address || data), where
        // attestation_domain_separator is the hardcoded utf-8 encoding of the string
        // "ZK_REAL_ATTESTATION", pbc_contract_address is the address of the contract that requested
        // the attestation and data is the actual data to be signed.
        // For the voting case it means that compute the digest of
        // "ZK_REAL_ATTESTATION" || privateVotingPbcAddress || _voteId || _votesFor || _votesAgainst ||
        // We use abi.encodePacked to ensure the bytes are encoded in the same manner as on PBC.
        return sha256(
            abi.encodePacked(
                "ZK_REAL_ATTESTATION",
                privateVotingPbcAddress,
                _voteId,
                _votesFor,
                _votesAgainst
            ));
    }
}

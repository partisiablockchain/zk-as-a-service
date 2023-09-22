//! # Secret Voting Contract
//!
//! The purpose of this specific secret voting contract is to demonstrate how PBC can be used as a
//! second layer to provide the ZK computation feature to an existing web3 application.
//!
//! See [the documentation](https://partisiablockchain.gitlab.io/documentation/PBC-as-a-second-layer/pbc-as-second-layer.html)
//! for an overview of the set-up.
//!
//! This example contract is designed in such a way that after a vote has ended a new one
//! automatically starts.
//!
//! The contract flow can be summarized in these steps:
//!
//! 1. Initialization of contract. The id of the first one is 1 and the list of results is empty.
//! 2. Users can cast their secret votes. ("false" is against, "true" is for).
//! 3. At any point can anyone start vote counting.
//! 4. Zk Computation sums yes votes and no votes, and output each as a separate variable.
//! 5. When computation is complete the contract will open the output variables.
//! 6. The result (number of yes votes, number of no votes and number of absent voters) is attested by computation nodes.
//! 7. The result is added to the list of historic votes, the vote id is incremented and all inputs are deleted.
//! 8. A new vote has begun, starting from step 3.

#[macro_use]
extern crate pbc_contract_codegen;
extern crate pbc_contract_common;

use std::fmt::Write;

use create_type_spec_derive::CreateTypeSpec;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::events::EventGroup;
use pbc_contract_common::signature::Signature;
use pbc_contract_common::zk::AttestationId;
use pbc_contract_common::zk::{CalculationStatus, SecretVarId, ZkInputDef, ZkState, ZkStateChange};
use pbc_traits::WriteRPC;
use pbc_zk::{Sbi1, SecretBinary};
use read_write_state_derive::ReadWriteState;

/// Structure representing the metadata attached to a secret variable in the ZK state,
/// indicating which type of variable it is.
/// For this voting example, a secret variable can either be a user inputted vote, or the result of
/// running the ZK computation, i.e. the final count of "yes" votes.
#[derive(ReadWriteState, Debug)]
#[repr(C)]
struct SecretVarMetadata {
    variable_type: SecretVarType,
}

#[derive(ReadWriteState, Debug, PartialEq)]
#[repr(u8)]
enum SecretVarType {
    Vote = 1,
    CountedYesVotes = 2,
}

/// Result of a vote after counting is complete.
///
/// Structure representing the result of a secret vote.
#[derive(ReadWriteState, CreateTypeSpec, Clone)]
struct VoteResult {
    /// The identifier for the vote the result is valid for.
    vote_id: u32,
    /// Number of votes cast in favor.
    votes_for: u32,
    /// Number of votes cast against.
    votes_against: u32,
    /// Proof of the vote result that can be validated on Ethereum contract.
    proof: Option<String>,
}

/// Structure representing the open state for private voting contract.
#[state]
struct ContractState {
    /// Id for the current vote.
    current_vote_id: u32,
    /// List of result for all votes that have been resolved.
    vote_results: Vec<VoteResult>,
}

/// Method for initializing the contract's state. To make the ids of the vote result match the ids
/// of the data attestations, the first vote has id 1.
#[init(zk = true)]
fn initialize(_ctx: ContractContext, _zk_state: ZkState<SecretVarMetadata>) -> ContractState {
    ContractState {
        current_vote_id: 1,
        vote_results: vec![],
    }
}

/// The bit size of the secret vote. A vote can either be 0 or 1, so a single bit is needed.
const BITLENGTH_OF_SECRET_VOTE_VARIABLES: u32 = 1;

/// A secret vote. False means against and true means for.
#[derive(CreateTypeSpec, SecretBinary)]
#[allow(dead_code)]
struct SecretVote {
    vote: Sbi1,
}

/// A voter can cast a secret vote using this function.
///
/// The function ensures that input only happens when there is no active ZK calculation in progress,
/// and that the voter has not already cast a vote.
///
/// The type of input is specified as the SecretVote struct defined above.
///
/// The ZkInputDef encodes that the secret vote must have size
/// [`BITLENGTH_OF_SECRET_VOTE_VARIABLES`].
#[zk_on_secret_input(shortname = 0x40, secret_type = "SecretVote")]
fn cast_vote(
    context: ContractContext,
    state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
) -> (
    ContractState,
    Vec<EventGroup>,
    ZkInputDef<SecretVarMetadata>,
) {
    // If calculation status is not Waiting, we cannot receive secret inputs. This could happen if
    // a ZK computation is in progress or if we are cleaning the state after such a computation.
    assert_eq!(
        zk_state.calculation_state,
        CalculationStatus::Waiting,
        "Vote casting must be done in Waiting state, but was {:?}",
        zk_state.calculation_state,
    );

    // Ensure that the account casting the vote has not already voted in this round.
    assert!(
        zk_state
            .secret_variables
            .iter()
            .chain(zk_state.pending_inputs.iter())
            .all(|v| v.owner != context.sender),
        "Each voter is only allowed to send one vote variable. Sender: {:?}",
        context.sender
    );
    // Build the ZK input definition, specifying the bit length of the vote, and the type of the
    // variable to be a vote.
    let input_def = ZkInputDef {
        seal: false,
        metadata: SecretVarMetadata {
            variable_type: SecretVarType::Vote,
        },
        expected_bit_lengths: vec![BITLENGTH_OF_SECRET_VOTE_VARIABLES],
    };
    // Return the state as is, no events and the input definition of the variable.
    (state, vec![], input_def)
}

/// Start counting of the secret votes top produce the vote result.
///
/// Counting can be started by anyone. The function ensures that the current status of the
/// calculation is neither active nor outputting.
///
/// Note: Any votes that have not been confirmed by the computation nodes are not included in the
/// count but will instead be included in the next vote iteration.
///
/// After starting the counting, the rest of the process (e.g. opening and attesting the result,
/// cleaning up old variables and starting a new vote) happens automatically.
#[action(shortname = 0x01, zk = true)]
fn start_vote_counting(
    _context: ContractContext,
    state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    // Ensure that we do not start counting votes if calculation status is not Waiting.
    // If the status is not waiting a computation may already be in progress or we are cleaning up
    // after a computation.
    assert_eq!(
        zk_state.calculation_state,
        CalculationStatus::Waiting,
        "Vote counting must start from Waiting state, but was {:?}",
        zk_state.calculation_state,
    );

    // Return the state unmodified, and no events. Request that the computation begins and define
    // metadata to be attached to the secret output variable.
    (
        state,
        vec![],
        vec![ZkStateChange::start_computation(vec![SecretVarMetadata {
            variable_type: SecretVarType::CountedYesVotes,
        }])],
    )
}

/// Automatically called when the computation is completed
///
/// The only thing we do is to instantly open/declassify the output variables.
#[zk_on_compute_complete]
fn open_yes_count_variable(
    _context: ContractContext,
    state: ContractState,
    _zk_state: ZkState<SecretVarMetadata>,
    output_variables: Vec<SecretVarId>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    // Immediately request that the output variable, i.e. the count of yes votes, is opened and
    // made public.
    (
        state,
        vec![],
        vec![ZkStateChange::OpenVariables {
            variables: output_variables,
        }],
    )
}

/// Automatically called when a variable is opened/declassified.
///
/// We can now read the for and against variables, and compute the result.
/// Once the result has been computed we request that the Zk nodes attest the result (i.e sign it)
/// and save it to this contracts open state.
#[zk_on_variables_opened]
fn build_and_attest_voting_result(
    _context: ContractContext,
    mut state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
    opened_variables: Vec<SecretVarId>,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    // Get the id of the variable that was opened after the computation was completed.
    let computation_result_variable_id = opened_variables.get(0).unwrap();
    // Build the result of the vote by getting the raw numbers from the opened variables and the
    // state.
    let vote_result = determine_result(&state, &zk_state, computation_result_variable_id);
    // Add the result to the open state. The result is still missing the proof.
    state.vote_results.push(vote_result.clone());
    // Return the tuple with the modified state, no events, and with a request that the computation
    // nodes sign the serialized bytes of the result.
    (
        state,
        vec![],
        vec![ZkStateChange::Attest {
            data_to_attest: serialize_result_as_big_endian(vote_result),
        }],
    )
}

/// Automatically called once all nodes have signed the data we requested.
///
/// Get the signatures for the attestation, formats them for EVM, and adds as proof on the result.
/// Then delete all variables from the old vote, set the id for the next one and set the
/// calculation status back to "waiting" so we can receive new secret voting inputs for the new
/// vote.
#[zk_on_attestation_complete]
fn save_attestation_on_result_and_start_next_vote(
    _context: ContractContext,
    mut state: ContractState,
    zk_state: ZkState<SecretVarMetadata>,
    attestation_id: AttestationId,
) -> (ContractState, Vec<EventGroup>, Vec<ZkStateChange>) {
    // Get ids of all secret variables, to delete all votes cast in the previous vote before
    // starting the next one.
    let variables_to_delete: Vec<SecretVarId> = zk_state
        .secret_variables
        .iter()
        .map(|x| x.variable_id)
        .collect();

    // Find the result of the vote that was just concluded. We want to store the proof on the result
    // so we need it to be mutable to update the proof field.
    let result = state
        .vote_results
        .iter_mut()
        .find(|r| r.vote_id == state.current_vote_id)
        .unwrap();

    // The signatures provided by the computation nodes can be found on the data attestation object
    // in the zk state. Find the attestation that has the same id as the one provided in the
    // arguments.
    let attestation = zk_state
        .data_attestations
        .iter()
        .find(|a| a.attestation_id == attestation_id)
        .unwrap();

    // Parse the signatures into a text format that can be used in an Eth transaction without
    // further data conversions. The format is an array of the signatures in hex encoding.
    let proof_of_result = format! {"[{}]", attestation
    .signatures
    .iter()
    .map(as_evm_string)
    .collect::<Vec<String>>()
    .join(", ")};

    // Save the proof on the result object for convenient retrieval.
    result.proof = Some(proof_of_result);
    // Increment the vote id.
    state.current_vote_id += 1;
    // Return the tuple with the new updated state, no events, and an update to notify the runtime
    // environment to delete the variables and set the calculation status to Waiting. This ensures
    // that the contract will accept secret votes for the next round.
    (
        state,
        vec![],
        vec![ZkStateChange::OutputComplete {
            variables_to_delete,
        }],
    )
}

/// Serialize the vote result into a binary format that matches the format used by ethereum's
/// abi.encodePacked() method, i.e. 4 32-bit unsigned integers encoded in big endian format.
fn serialize_result_as_big_endian(result: VoteResult) -> Vec<u8> {
    let mut output: Vec<u8> = vec![];
    result
        .vote_id
        .rpc_write_to(&mut output)
        .expect("Unable to serialize vote_id");
    result
        .votes_for
        .rpc_write_to(&mut output)
        .expect("Unable to serialize votes_for");
    result
        .votes_against
        .rpc_write_to(&mut output)
        .expect("Unable to serialize votes_against");
    output
}

/// Encode a [`Signature`] as a hex-string representation of a signature that can be parsed by the
/// EVM.
///
/// To make the signature parseable by the EVM, add 27 to the recovery id. The output should be 64
/// chars of the encoded r value, followed by 64 chars of the encoded s value and finally 2 chars
/// of the encoded recovery id. The entire string is prepended with "0x".
fn as_evm_string(signature: &Signature) -> String {
    // Ethereum expects that the recovery id has value 0x1B or 0x1C, but the algorithm used by PBC
    // outputs 0x00 or 0x01. Add 27 to the recovery id to ensure it has an expected value, and
    // format as a hexidecimal string.
    let recovery_id = signature.recovery_id + 27;
    let recovery_id = format!("{recovery_id:02x}");
    // The r value is 32 bytes, i.e. a string of 64 characters when represented in hexidecimal.
    let mut r = String::with_capacity(64);
    // For each byte in the r value format is a hexidecimal string of length 2 to ensure zero
    // padding, and write it to the output string defined above.
    for byte in signature.value_r {
        write!(r, "{byte:02x}").unwrap();
    }
    // Do the same for the s value.
    let mut s = String::with_capacity(64);
    for byte in signature.value_s {
        write!(s, "{byte:02x}").unwrap();
    }
    // Combine the three values into a single string, prepended with "0x".
    format!("0x{r}{s}{recovery_id}")
}

/// Determines the result of the vote in raw numbers, by reading the number of yes votes and
/// deriving the number of no votes.
fn determine_result(
    state: &ContractState,
    zk_state: &ZkState<SecretVarMetadata>,
    computation_result_variable_id: &SecretVarId,
) -> VoteResult {
    // Read the opened result of the ZK computation, which is a count of how many yes votes were
    // cast It is stored as an unsigned 32 bit integer in little endian format.
    let votes_for = read_variable_u32_le(zk_state, computation_result_variable_id);
    // Count the number of secret variables of type Vote, to get total number of cast votes.
    let total_votes = zk_state
        .secret_variables
        .iter()
        .filter(|x| x.metadata.variable_type == SecretVarType::Vote)
        .count() as u32;
    // Calculate the number of no votes as the number of yes votes subtracted from the total votes.
    let votes_against = total_votes - votes_for;
    // Build the vote result from the numbers and set the proof to None as we don't have it yet.
    VoteResult {
        vote_id: state.current_vote_id,
        votes_for,
        votes_against,
        proof: None,
    }
}

/// Reads a variable's data as an u32.
fn read_variable_u32_le(
    zk_state: &ZkState<SecretVarMetadata>,
    yes_count_variable_id: &SecretVarId,
) -> u32 {
    // Get the actual variable from state.
    let yes_count_variable = zk_state.get_variable(*yes_count_variable_id).unwrap();
    // Defined buffer to save the variable data in.
    let mut buffer = [0u8; 4];
    // Copy the variable data to the buffer.
    buffer.copy_from_slice(yes_count_variable.data.as_ref().unwrap().as_slice());
    // Cast the variable bytes to a u32, specifying that the bytes are ordered in little endian.
    <u32>::from_le_bytes(buffer)
}

# Diem Safety Rules

## Overview

Diem Safety Rules (LSR) ensures that a validator running LSR will not become byzantine. This translates into the property that a committed transaction is irreversible. This property is known as "no-forks" and effectively means that consensus rounds are strictly linear and storage is append-only.  A fork occurs when two ledgers or committed blocks, LI1 and LI2, do not have a linear relationship, in other words, neither LI1 is a prefix of LI2 nor LI2 is a prefix of LI1.

This specification builds on top of the [Consensus specification](../consensus/spec.md) and defines how to separate out the [Consensus safety rules](../consensus/spec.md#safety-rules) into a minimal software component that ensures that even a Byzantine validator would be unable to violate the "no-forks" property so long as LSR has not been compromised. In effect, this is the validator's consensus trusted computing base:

> a small amount of software and hardware that security depends on and that we distinguish from a much larger amount that can misbehave without affecting security. - Lampson, Authentication in Distributed Systems: Theory and Practice

## Properties

By maintaining the following properties, LSR ensures that consensus will observe the "no-forks" guarantee:

1. A validator can only vote on proposals newer than those they have already voted on as defined by linearly increasing epochs and rounds.
2. A validator can only vote on a proposal if it extends from a certified block, or a proposal and its executed state that a quorum of voters have seen.
3. A validator can only vote on a proposal if the output of that proposal extends its parent's executed state.
4. The key used to sign proposals is used solely by LSR and in a way that would not violate the previous properties.
5. Additionally, when combined with Diem Execution Correctness (LEC), ensures that the transactions are faithfully executed and committed.

## Terms

Name                 | Description
-------------------- | -------------------------------------------------------------------------------------
`validator`          | An entity that participates in consensus
`quorum_cert`        | Contains a quorum of votes on a specific round and its parent round
`ledger_info`        | A committed state to the blockchain that contains the signatures from a `quorum_cert`
`consensus_key`      | An Ed25519 private key used to sign all consensus messages, including proposals
`validator_verifier` | An aggregation of the Ed25519 public keys for all validators
`execution_key`      | An Ed25519 private key, optionally, used by the executor to certify execution state
`last_voted_round`   | The last `round` that a validator submitted a vote
`preferred_round`    | The highest parent `round` from any incoming quorum certificate
`epoch`              | Distinguishes sets of `round`s and potentially different `validator_verifier`s
`waypoint`           | An epoch ending `ledger_info` that contains a `validator_verifier`

## Design and Deployment

LSR design and deployment embody the following principles:

* Isolation from other validator services including consensus and ledger storage, hence isolating simple logic, code, and dependencies from more complex.
* Code and protocol integrity for the LSR implementation, assumption that LSR's software and environment perform the operations faithfully.
* Reliance on [Secure Storage](secure_storage.md) to store the Safety Rule state and secure the consensus key
* That the provided `waypoint` points to an accurate state in the blockchain
* (Optional) higher levels of trust for execution via a signature when running with [LEC](execution_correctness/execution_correctness_specification.md)
* Attempt to validate correct consensus protocol execution as much as possible

## Interface

The SafetyRules interface largely builds upon existing data types mostly contained within the [Consensus specification](../consensus/spec.md). Each of the data types have a well defined BCS representation allowing LSR to be hosted as a component running in the same process as consensus or independently in its own domain, such as a process or within a secure enclave.

Consensus communicates to LSR through the following interface, which also defines the minimal set of functions required to operate an external LSR:

```rust
/// Interface for SafetyRules
pub trait TSafetyRules {
    /// Provides the internal state of SafetyRules for monitoring / debugging purposes. This does
    /// not include sensitive data like private keys.
    fn consensus_state(&mut self) -> Result<ConsensusState, Error>;

    /// Initialize SafetyRules using an Epoch ending LedgerInfo, this should map to what was
    /// provided in consensus_state. It will be used to initialize the ValidatorSet.
    /// This uses a EpochChangeProof because there's a possibility that consensus migrated to a
    /// new epoch but SafetyRules did not.
    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error>;

    /// Attempts to vote for a given proposal following the voting rules.
    fn construct_and_sign_vote(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error>;

    /// As the holder of the private key, SafetyRules also signs proposals or blocks.
    /// A Block is a signed BlockData along with some additional metadata.
    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error>;

    /// As the holder of the private key, SafetyRules also signs what is effectively a
    /// timeout message. This returns the signature for that timeout message.
    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error>;
}
```

Consensus can query the current internal state of SafetyRules via the ```consensus_state``` function:

```rust
pub struct ConsensusState {
    epoch: u64,
    last_voted_round: Round,
    preferred_round: Round,
    waypoint: Waypoint,
    in_validator_set: bool,
}
```

LSR retains the following internal state:

```rust
pub struct SafetyRules {
    /// An interface into a secure storage implementation
    persistent_storage: PersistentSafetyStorage,
    /// Key used to verify authenticity of LEC execution results
    execution_public_key: Option<Ed25519PublicKey>,
    /// Signing key to vote on proposals and sign timeouts and proposals
    validator_signer: Option<ValidatorSigner>,
    /// validator_verifier to ensure correctly formed proposals and certificates
    epoch_state: Option<EpochState>,
}
```

LSR may emit one of the following errors:

```rust
pub enum Error {
    #[error("Provided epoch, {0}, does not match expected epoch, {1}")]
    IncorrectEpoch(u64, u64),
    #[error("Provided round, {0}, is incompatible with last voted round, {1}")]
    IncorrectLastVotedRound(u64, u64),
    #[error("Provided round, {0}, is incompatible with preferred round, {1}")]
    IncorrectPreferredRound(u64, u64),
    #[error("Unable to verify that the new tree extneds the parent: {0}")]
    InvalidAccumulatorExtension(String),
    #[error("Invalid EpochChangeProof: {0}")]
    InvalidEpochChangeProof(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("No next_epoch_state specified in the provided Ledger Info")]
    InvalidLedgerInfo,
    #[error("Invalid proposal: {}", {0})]
    InvalidProposal(String),
    #[error("Invalid QC: {}", {0})]
    InvalidQuorumCertificate(String),
    #[error("{0} is not set, SafetyRules is not initialized")]
    NotInitialized(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Vote proposal missing expected signature")]
    VoteProposalSignatureNotFound,
}
```

## State Machine Protocol

### Initialization -- `initialize`

Consensus separates a series of round into distinct epochs. Each epoch defines its own `validator_verifier` and has its own independent set of rounds, beginning with `0`.

LSR begins operation by receiving an `EpochChangeProof` from consensus.  This contains a set of `ledger_info`s inclusive of the current LSR `waypoint`. LSR should expect an initialization message at any time, but a fresh LSR instance *must* receive one in order to begin executing. Hence there are three states wherein LSR will receive this message:

* LSR has yet to be initialized
* Validator has been restarted and consnesus has just begun operating
* Consensus enters a new epoch

The outcomes from receiving this message depend on whether or not LSR is entering a new `epoch`, in other words, the final `ledger_info` within the `EpochChangeProof` is greater than the current epoch within LSR persistent store.

* If epochs are the same, then LSR will configure the `validator_verifier` and obtain its correct `consensus_key` given its public key within the `validator_verifier`.
* If there is a new epoch, then LSR will configure the `validator_verifier` and obtain its correct `consensus_key` given its public key within the `validator_verifier` as well as update its `waypoint` to point to the latest `ledger_info`, update its `epoch` to point to this new epoch, and finally reset both `last_voted_round` and `preferred_round` to `0`.
* A previous epoch is impossible otherwise this would violate the `waypoint` verification performed earlier.

**Notes**:

1. If LSR finds that it is unable to find its expected `consensus_key` in storage it will error.
2. LSR depends on storage to have been initialized with `epoch`, `last_voted_round`, `preferred_round`, and `waypoint`. At Genesis, these are set to `0`,`0`, `0`, and the `ledger_info` after Genesis.
3. LSR can leverage the `validator_verifier` to identify the current expected `consensus_key` and **must** not participate in any other operation unless LSR has access to this key.

### Voting -- `construct_and_sign_vote`

Consensus relies on LSR to sign votes. In order to ensure these properties, Consensus provides a `VoteProposal` that may have been signed by LEC if enabled.  LSR verifies and updates state via the properties mentioned above (1, 2, 3, 5), then signs a `Vote` derived from the `VoteProposal` and returns it to consensus.

Prior to ensuring any properties, LSR must first check if LSR has a valid `consensus_key` for signing the proposal and that the proposal's `epoch` matches the current `epoch`.

#### Property 1

> A validator can only vote on proposals newer than those they have already voted on as defined by linearly increasing epochs and rounds.

LSR enforces this by keeping track of `last_voted_round`, this is updated immediately prior to LSR voting on a proposal. Therefore LSR can never vote on the same round twice or an earlier round than the current.

#### Property 2

> A validator can only vote on a proposal if it extends from a proposal that a quorum of voters have seen.

LSR enforces this by ensuring that the `quorum_cert` associated with a proposal is properly constructed, is correctly signed leveraging the `validator_verifier`, and that the parent's round within the `quorum_cert` is equal to or greater than the `preferred_round`.  The certified state of parent round within the `quorum_cert` is guaranteed to have been seen by a quorum of validators and hence serves as a bookmark to not backtrack to earlier states.  Because of consensus 3-chain commit rule, violations of this could break safety.

LSR updates the persistent storage `preferred_round` to the maximum of the `quorum_cert`'s parent's round or the current stored `preferred_round`.

#### Property 3

> A validator can only vote on a proposal if the output of that proposal extends its parent's output.

Each `Vote` contains the executed state output, the resulting root hash of the blockchain ledger after executing the proposal. In order to verify that this extends the parent block, a `VoteProposal` contains an `AccumulatorExtensionProof` which is a merkle-proof showing how the current state can be derived from the parent's state, hence ensuring that the ledger is append-only.

#### Property 4

> Additionally, when combined with Diem Execution Correctness (LEC), ensures that the transactions are faithfully executed and committed to the blockchain.

The validator node may be configured to leverage an additional process known as the Diem Execution Correctness (LEC). LEC is another TCB that runs in a more trusted fashion than consensus itself. Because consensus forwards all messages to LSR, LEC constructs and signs the `VoteProposal` with the `execution_key`. This informs LSR that the contents have not been tampered by processes running in a less trusted environment than LEC including consensus.

### Proposing -- `sign_proposal`

As part of property 4, LSR must be the only holder of the `consensus_key` and therefore signs all proposals. In practice, LSR need not verify any property of the proposal except that it is indeed a proposal. To mitigate bugs and similar issues, additional checks can be performed:

* the `author` is the LSR `validator`
* the LSR `epoch` is equal to the proposal's epoch
* the LSR `last_voted_round` is lower than the proposal's round
* the propoal respects the `preferred_round`
* the `consensus_key` is available

If the proposal's `quorum_cert` indicates a more recent `preferred_round`, the data in LSR's persistent storage is updated.

### Timing out -- `sign_timeout`

As part of property 4, LSR must be the only holder of the `consensus_key` and therefore signs all timeouts. LSR must requires the following to sign a timeout:

* the `consensus_key` is available
* the LSR `epoch` is equal to the timeout's epoch
* the LSR `last_voted_round` is lower than or equal to the timeout's round

LSR updates the persistent storage `last_voted_round` to the maximum of the timeout and the current `last_voted_round` within persistent storage.

Finally, LSR signs the timeout and returns a signed timeout to consensus.

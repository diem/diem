# Overview

## Description of files

- **Vote.move** : actual Voting module which manages Ballots, Votes and Approval Policies for Ballots
- **CreateValidator.move** : Sample module which builds on top of `Vote` module
- **init_approval_policy.move** : Script to intialize various items to enable Voting in Global Storage
- **create_ballot_and_vote.move** : Sample Script which shows proposal creation and a single vote submission for a CreateValidator transaction

## Global Storage:

We store the following 3 structs ("has key") in the Global Storage:

- ApprovalPolicies: This is a "map" of proposal_types to their approval policies. This is stored under the root address(0xA550C18) eg

```
[
    {proposal_type: "create_validator_owner", allowed_voters: ["alice", "bob", "carol"], num_votes_required: 2},
    {proposal_type: "create_dd", allowed_voters: ["bob", "carol", "dave"], num_votes_required: 1}
]
```

- Ballots<Proposal>: This stores a list of ballots for each proposal type. This is stored under each proposer's address.

- BallotCounter: A counter which is stored under the proposers address and gets incremented everytime they create a new ballot.

## Initialization

- We need to initialize the ApprovalPolicies and BallotCounter before they can be used in operations.
  - See init_approval_policy.move

## Typical flow

- Someone creates a new proposal using the approprite function defined in that module. Eg: `CreateValidatorAccount::propose_create_validator_account(&account, @0x1234, b"abcd", b"abcd");`

- Voters send votes using approprite function defined in that module. Eg:

```
    CreateValidatorAccount::vote_on_create_validator_account(
        &account,
        0,
        @0x2,
        @0x1234,
        b"abcd",
        b"abcd",
    );
```

- If enough votes are gathered, vote_on_create_validator_account goes ahead and creates the validator account (and maybe calls gc function to remove that ballot)

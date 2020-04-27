# Libra Config Manager

The Libra Config Manager provides a tool for end-to-end management of the Libra
blockchain from genesis to maintenance. The functionality of the tool is
dictated by the organization of nodes within the system:

* An association account that maintains the set of node owners and validator
  operators.
* Node owners (NO) that have accounts on the blockchain. These accounts contain
  a validator configuration and specify a validator operator.
* Validator operators (VO) that have accounts on the blockchain. These
  accounts have the ability to manipulate validator configuration.

## Generating Genesis

The process for starting organization of The planned and current functionality includes:

* Initialization ceremony
  * Association sets up a secure-backend for data uploads, `association drive`.
    The association then distributes credentials for each node owner and
    validator operator.
  * The association generates its `association key` and shares the public key
    to the association drive.
  * Each NO will generate a private `owner key` and share the public key to the
    association drive.
  * Each VO will generate a private `operator key` and share the public key to
    the association drive.
* Validator initialization
  * Each NO will select a VO and submit this as a transaction signed by their
    `owner key` and uploads it to the association drive..
  * For each validator supported by a VO, the VO will generate network and
    consensus keys as well as network addresses for fullnode and validator
    endpoints. The VO will generate a transaction containing this data signed
    by their `operator key` and uploads it to the association drive.
* Genesis
  * Each VO will download the accumulated data to produce a genesis.blob
  * Association will download the accumulated data to produce both a
    genesis.blob and genesis waypoint.
* Starting
  * Association publishes the data associated with genesis, the genesis.blob,
    and the genesis waypoint.
  * Each VO downloads the genesis waypoint provided by the association and
    inserts it into their Libra instance(s) and starts them.
  * Upon a quorum of validators coming online, the blockchain will begin
    processing transactions.

Notes:
* This describes a process for instantiating organization that has yet to be
  specified but extends from the current state of the Libra Testnet.
* The implementation as described has yet to be fully implemented in Move,
  hence, this tool maps to the current state.

## Requirements

Each individual instance, NO or VO, should have access to a secure storage
solution. Those leveraging Libra Secure Storage can directly use this tool,
those that do not will need to provide their own tooling.

## The Tools

While this is compiled as a single binary it provides several different facilities:

* A means for bootstrapping an identity mapped to a local configuration file.
  The identities are used to interact with local and remote secure storages.
* Retrieving and submitting node operator, validator operator, and validator
  configuration -- this is from a local secure storage to a remote secure
  storage -- leveraging the identity tool.
* Converting a genesis configuration and a secure storage into a genesis.blob /
  genesis waypoint.

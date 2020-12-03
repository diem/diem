# Diem Operational Tool

The `diem-operational-tool` provides a tool for operators to perform maintenance functions
on the Diem blockchain post-genesis.  The functionality of the tool is
dictated by the organization of nodes within the system:

* Validator owners (OW) that have accounts on the blockchain. These accounts contain
  a validator configuration and specify a validator operator.
* Validator operators (OP) that have accounts on the blockchain. These
  accounts have the ability to update validator configuration.

### Important Notes

* A namespace in Vault is represented as a subdirectory for secrets and a
  prefix followed by `__` for transit, e.g., `namespace__`.
* A namespace in GitHub is represented by a subdirectory
* The GitHub repository and repository owner translate into the following url:
  `https://github.org/REPOSITORY_OWNER/REPOSITORY`
* The owner-address is intentionally set as all 0s as it is unused at this
  point in time.

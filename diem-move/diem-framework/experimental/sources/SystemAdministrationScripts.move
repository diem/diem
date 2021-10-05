/// This module contains Diem Framework script functions to administer the
/// network outside of validators and validator operators.
module DiemFramework::SystemAdministrationScripts {
    use DiemFramework::DiemConsensusConfig;
    use ExperimentalFramework::ExperimentalVersion;
    use DiemFramework::DiemVMConfig;
    use DiemFramework::SlidingNonce;

    ///  # Summary
    /// Updates the Diem major version that is stored on-chain and is used by the VM.  This
    /// transaction can only be sent from the Diem Root account.
    ///
    /// # Technical Description
    /// Updates the `DiemVersion` on-chain config and emits a `DiemConfig::NewEpochEvent` to trigger
    /// a reconfiguration of the system. The `major` version that is passed in must be strictly greater
    /// than the current major version held on-chain. The VM reads this information and can use it to
    /// preserve backwards compatibility with previous major versions of the VM.
    ///
    /// # Parameters
    /// | Name            | Type     | Description                                                                |
    /// | ------          | ------   | -------------                                                              |
    /// | `account`       | `signer` | Signer of the sending account. Must be the Diem Root account.              |
    /// | `sliding_nonce` | `u64`    | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction. |
    /// | `major`         | `u64`    | The `major` version of the VM to be used from this transaction on.         |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                  | Description                                                                                |
    /// | ----------------           | --------------                                | -------------                                                                              |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                | A `SlidingNonce` resource is not published under `account`.                                |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                | The `sliding_nonce` is too far in the future.                                              |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`       | The `sliding_nonce` has been previously recorded.                                          |
    /// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::EDIEM_ROOT`                   | `account` is not the Diem Root account.                                                    |
    /// | `Errors::INVALID_ARGUMENT` | `DiemVersion::EINVALID_MAJOR_VERSION_NUMBER`  | `major` is less-than or equal to the current major version stored on-chain.                |

    public(script) fun update_diem_version(account: signer, sliding_nonce: u64, major: u64) {
        SlidingNonce::record_nonce_or_abort(&account, sliding_nonce);
        ExperimentalVersion::set(&account, major)
    }

    /// # Summary
    /// Updates the gas constants stored on chain and used by the VM for gas
    /// metering. This transaction can only be sent from the Diem Root account.
    ///
    /// # Technical Description
    /// Updates the on-chain config holding the `DiemVMConfig` and emits a
    /// `DiemConfig::NewEpochEvent` to trigger a reconfiguration of the system.
    ///
    /// # Parameters
    /// | Name                                | Type     | Description                                                                                            |
    /// | ------                              | ------   | -------------                                                                                          |
    /// | `account`                           | `signer` | Signer of the sending account. Must be the Diem Root account.                                          |
    /// | `sliding_nonce`                     | `u64`    | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction.                             |
    /// | `global_memory_per_byte_cost`       | `u64`    | The new cost to read global memory per-byte to be used for gas metering.                               |
    /// | `global_memory_per_byte_write_cost` | `u64`    | The new cost to write global memory per-byte to be used for gas metering.                              |
    /// | `min_transaction_gas_units`         | `u64`    | The new flat minimum amount of gas required for any transaction.                                       |
    /// | `large_transaction_cutoff`          | `u64`    | The new size over which an additional charge will be assessed for each additional byte.                |
    /// | `intrinsic_gas_per_byte`            | `u64`    | The new number of units of gas that to be charged per-byte over the new `large_transaction_cutoff`.    |
    /// | `maximum_number_of_gas_units`       | `u64`    | The new maximum number of gas units that can be set in a transaction.                                  |
    /// | `min_price_per_gas_unit`            | `u64`    | The new minimum gas price that can be set for a transaction.                                           |
    /// | `max_price_per_gas_unit`            | `u64`    | The new maximum gas price that can be set for a transaction.                                           |
    /// | `max_transaction_size_in_bytes`     | `u64`    | The new maximum size of a transaction that can be processed.                                           |
    /// | `gas_unit_scaling_factor`           | `u64`    | The new scaling factor to use when scaling between external and internal gas units.                    |
    /// | `default_account_size`              | `u64`    | The new default account size to use when assessing final costs for reads and writes to global storage. |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                | Description                                                                                |
    /// | ----------------           | --------------                              | -------------                                                                              |
    /// | `Errors::INVALID_ARGUMENT` | `DiemVMConfig::EGAS_CONSTANT_INCONSISTENCY` | The provided gas constants are inconsistent.                                               |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`              | A `SlidingNonce` resource is not published under `account`.                                |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`              | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`              | The `sliding_nonce` is too far in the future.                                              |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`     | The `sliding_nonce` has been previously recorded.                                          |
    /// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::EDIEM_ROOT`                 | `account` is not the Diem Root account.                                                    |
    public(script) fun set_gas_constants(
        dr_account: signer,
        sliding_nonce: u64,
        global_memory_per_byte_cost: u64,
        global_memory_per_byte_write_cost: u64,
        min_transaction_gas_units: u64,
        large_transaction_cutoff: u64,
        intrinsic_gas_per_byte: u64,
        maximum_number_of_gas_units: u64,
        min_price_per_gas_unit: u64,
        max_price_per_gas_unit: u64,
        max_transaction_size_in_bytes: u64,
        gas_unit_scaling_factor: u64,
        default_account_size: u64,
    ) {
        SlidingNonce::record_nonce_or_abort(&dr_account, sliding_nonce);
        DiemVMConfig::set_gas_constants(
                &dr_account,
                global_memory_per_byte_cost,
                global_memory_per_byte_write_cost,
                min_transaction_gas_units,
                large_transaction_cutoff,
                intrinsic_gas_per_byte,
                maximum_number_of_gas_units,
                min_price_per_gas_unit,
                max_price_per_gas_unit,
                max_transaction_size_in_bytes,
                gas_unit_scaling_factor,
                default_account_size,
        )
    }

    ///  # Summary
    /// Initializes the Diem consensus config that is stored on-chain.  This
    /// transaction can only be sent from the Diem Root account.
    ///
    /// # Technical Description
    /// Initializes the `DiemConsensusConfig` on-chain config to empty and allows future updates from DiemRoot via
    /// `update_diem_consensus_config`. This doesn't emit a `DiemConfig::NewEpochEvent`.
    ///
    /// # Parameters
    /// | Name            | Type      | Description                                                                |
    /// | ------          | ------    | -------------                                                              |
    /// | `account`       | `signer` | Signer of the sending account. Must be the Diem Root account.               |
    /// | `sliding_nonce` | `u64`     | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction. |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                  | Description                                                                                |
    /// | ----------------           | --------------                                | -------------                                                                              |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                | A `SlidingNonce` resource is not published under `account`.                                |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                | The `sliding_nonce` is too far in the future.                                              |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`       | The `sliding_nonce` has been previously recorded.                                          |
    /// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::EDIEM_ROOT`                   | `account` is not the Diem Root account.                                                    |

    public(script) fun initialize_diem_consensus_config(account: signer, sliding_nonce: u64) {
        SlidingNonce::record_nonce_or_abort(&account, sliding_nonce);
        DiemConsensusConfig::initialize(&account);
    }

    ///  # Summary
    /// Updates the Diem consensus config that is stored on-chain and is used by the Consensus.  This
    /// transaction can only be sent from the Diem Root account.
    ///
    /// # Technical Description
    /// Updates the `DiemConsensusConfig` on-chain config and emits a `DiemConfig::NewEpochEvent` to trigger
    /// a reconfiguration of the system.
    ///
    /// # Parameters
    /// | Name            | Type          | Description                                                                |
    /// | ------          | ------        | -------------                                                              |
    /// | `account`       | `signer`      | Signer of the sending account. Must be the Diem Root account.              |
    /// | `sliding_nonce` | `u64`         | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction. |
    /// | `config`        | `vector<u8>`  | The serialized bytes of consensus config.                                  |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                  | Description                                                                                |
    /// | ----------------           | --------------                                | -------------                                                                              |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                | A `SlidingNonce` resource is not published under `account`.                                |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                | The `sliding_nonce` is too far in the future.                                              |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`       | The `sliding_nonce` has been previously recorded.                                          |
    /// | `Errors::REQUIRES_ADDRESS` | `CoreAddresses::EDIEM_ROOT`                   | `account` is not the Diem Root account.                                                    |

    public(script) fun update_diem_consensus_config(account: signer, sliding_nonce: u64, config: vector<u8>) {
        SlidingNonce::record_nonce_or_abort(&account, sliding_nonce);
        DiemConsensusConfig::set(&account, config)
    }
}

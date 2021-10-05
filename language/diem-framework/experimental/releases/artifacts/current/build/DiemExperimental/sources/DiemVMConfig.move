/// This module defines structs and methods to initialize VM configurations,
/// including different costs of running the VM.
module DiemFramework::DiemVMConfig {
    use DiemFramework::DiemConfig::{Self, DiemConfig};
    use DiemFramework::DiemTimestamp;
    use DiemFramework::Roles;
    use Std::Errors;

    /// The provided gas constants were inconsistent.
    const EGAS_CONSTANT_INCONSISTENCY: u64 = 0;

    /// The struct to hold config data needed to operate the DiemVM.
    struct DiemVMConfig has copy, drop, store {
        /// Cost of running the VM.
        gas_schedule: GasSchedule,
    }

    /// The gas schedule keeps two separate schedules for the gas:
    /// * The instruction_schedule: This holds the gas for each bytecode instruction.
    /// * The native_schedule: This holds the gas for used (per-byte operated over) for each native
    ///   function.
    /// A couple notes:
    /// 1. In the case that an instruction is deleted from the bytecode, that part of the cost schedule
    ///    still needs to remain the same; once a slot in the table is taken by an instruction, that is its
    ///    slot for the rest of time (since that instruction could already exist in a module on-chain).
    /// 2. The initialization of the module will publish the instruction table to the diem root account
    ///    address, and will preload the vector with the gas schedule for instructions. The VM will then
    ///    load this into memory at the startup of each block.
    struct GasSchedule has copy, drop, store {
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
        gas_constants: GasConstants,
    }

    struct GasConstants has copy, drop, store {
        /// The cost per-byte read from global storage.
        global_memory_per_byte_cost: u64,

        /// The cost per-byte written to storage.
        global_memory_per_byte_write_cost: u64,

        /// The flat minimum amount of gas required for any transaction.
        /// Charged at the start of execution.
        min_transaction_gas_units: u64,

        /// Any transaction over this size will be charged an additional amount per byte.
        large_transaction_cutoff: u64,

        /// The units of gas to be charged per byte over the `large_transaction_cutoff` in addition to
        /// `min_transaction_gas_units` for transactions whose size exceeds `large_transaction_cutoff`.
        intrinsic_gas_per_byte: u64,

        /// ~5 microseconds should equal one unit of computational gas. We bound the maximum
        /// computational time of any given transaction at roughly 20 seconds. We want this number and
        /// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
        /// MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
        /// NB: The bound is set quite high since custom scripts aren't allowed except from predefined
        /// and vetted senders.
        maximum_number_of_gas_units: u64,

        /// The minimum gas price that a transaction can be submitted with.
        min_price_per_gas_unit: u64,

        /// The maximum gas unit price that a transaction can be submitted with.
        max_price_per_gas_unit: u64,

        max_transaction_size_in_bytes: u64,
        gas_unit_scaling_factor: u64,
        default_account_size: u64,
    }

    /// Initialize the table under the diem root account
    public fun initialize(
        dr_account: &signer,
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
    ) {
        DiemTimestamp::assert_genesis();

        // The permission "UpdateVMConfig" is granted to DiemRoot [[H11]][PERMISSION].
        Roles::assert_diem_root(dr_account);

        let gas_constants = GasConstants {
            global_memory_per_byte_cost: 4,
            global_memory_per_byte_write_cost: 9,
            min_transaction_gas_units: 600,
            large_transaction_cutoff: 600,
            intrinsic_gas_per_byte: 8,
            maximum_number_of_gas_units: 4000000,
            min_price_per_gas_unit: 0,
            max_price_per_gas_unit: 10000,
            max_transaction_size_in_bytes: 4096,
            gas_unit_scaling_factor: 1000,
            default_account_size: 800,
        };

        DiemConfig::publish_new_config(
            dr_account,
            DiemVMConfig {
                gas_schedule: GasSchedule {
                    instruction_schedule,
                    native_schedule,
                    gas_constants,
                }
            },
        );
    }
    spec initialize {
        let gas_constants = GasConstants {
            global_memory_per_byte_cost: 4,
            global_memory_per_byte_write_cost: 9,
            min_transaction_gas_units: 600,
            large_transaction_cutoff: 600,
            intrinsic_gas_per_byte: 8,
            maximum_number_of_gas_units: 4000000,
            min_price_per_gas_unit: 0,
            max_price_per_gas_unit: 10000,
            max_transaction_size_in_bytes: 4096,
            gas_unit_scaling_factor: 1000,
            default_account_size: 800,
        };

        /// Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemTimestamp::AbortsIfNotGenesis;
        include DiemConfig::PublishNewConfigAbortsIf<DiemVMConfig>;
        include DiemConfig::PublishNewConfigEnsures<DiemVMConfig> {
            payload: DiemVMConfig {
                gas_schedule: GasSchedule {
                    instruction_schedule,
                    native_schedule,
                    gas_constants,
                }
            }};
    }

    public fun set_gas_constants(
        dr_account: &signer,
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
        DiemTimestamp::assert_operating();
        Roles::assert_diem_root(dr_account);
        assert(
            min_price_per_gas_unit <= max_price_per_gas_unit,
            Errors::invalid_argument(EGAS_CONSTANT_INCONSISTENCY)
        );
        assert(
            min_transaction_gas_units <= maximum_number_of_gas_units,
            Errors::invalid_argument(EGAS_CONSTANT_INCONSISTENCY)
        );

        let config = DiemConfig::get<DiemVMConfig>();
        let gas_constants = &mut config.gas_schedule.gas_constants;

        gas_constants.global_memory_per_byte_cost       = global_memory_per_byte_cost;
        gas_constants.global_memory_per_byte_write_cost = global_memory_per_byte_write_cost;
        gas_constants.min_transaction_gas_units         = min_transaction_gas_units;
        gas_constants.large_transaction_cutoff          = large_transaction_cutoff;
        gas_constants.intrinsic_gas_per_byte            = intrinsic_gas_per_byte;
        gas_constants.maximum_number_of_gas_units       = maximum_number_of_gas_units;
        gas_constants.min_price_per_gas_unit            = min_price_per_gas_unit;
        gas_constants.max_price_per_gas_unit            = max_price_per_gas_unit;
        gas_constants.max_transaction_size_in_bytes     = max_transaction_size_in_bytes;
        gas_constants.gas_unit_scaling_factor           = gas_unit_scaling_factor;
        gas_constants.default_account_size              = default_account_size;

        DiemConfig::set(dr_account, config);
    }
    spec set_gas_constants {
        include DiemTimestamp::AbortsIfNotOperating;
        /// No one can update DiemVMConfig except for the Diem Root account [[H11]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};
        include DiemConfig::SetAbortsIf<DiemVMConfig>{account: dr_account };
        aborts_if min_price_per_gas_unit > max_price_per_gas_unit with Errors::INVALID_ARGUMENT;
        aborts_if min_transaction_gas_units > maximum_number_of_gas_units with Errors::INVALID_ARGUMENT;
        let config = DiemConfig::spec_get_config<DiemVMConfig>();
        ensures DiemConfig::spec_is_published<DiemVMConfig>();
        ensures DiemConfig::get<DiemVMConfig>() == DiemVMConfig {
            gas_schedule: GasSchedule {
                instruction_schedule: config.gas_schedule.instruction_schedule,
                native_schedule: config.gas_schedule.native_schedule,
                gas_constants: GasConstants {
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
                    },
                }
            };
        ensures old(DiemConfig::spec_has_config()) == DiemConfig::spec_has_config();
    }

    spec module { } // Switch documentation context to module level.

    /// # Initialization

    spec module {
        invariant [suspendable] DiemTimestamp::is_operating() ==> DiemConfig::spec_is_published<DiemVMConfig>();
    }

    /// # Access Control

    /// The permission "UpdateVMConfig" is granted to DiemRoot [[H11]][PERMISSION].
    spec module {
        invariant [suspendable] forall addr: address
            where exists<DiemConfig<DiemVMConfig>>(addr): addr == @DiemRoot;

        invariant update [suspendable] old(DiemConfig::spec_is_published<DiemVMConfig>())
            && DiemConfig::spec_is_published<DiemVMConfig>()
            && old(DiemConfig::get<DiemVMConfig>()) != DiemConfig::get<DiemVMConfig>()
                ==> Roles::spec_signed_by_diem_root_role();
    }

    // TODO: The following is the old style spec, which can removed later.
    /// No one can update DiemVMConfig except for the Diem Root account [[H11]][PERMISSION].
    spec schema DiemVMConfigRemainsSame {
        ensures old(DiemConfig::spec_is_published<DiemVMConfig>()) ==>
            global<DiemConfig<DiemVMConfig>>(@DiemRoot) ==
                old(global<DiemConfig<DiemVMConfig>>(@DiemRoot));
    }
    spec module {
        apply DiemVMConfigRemainsSame to * except set_gas_constants;
    }
}

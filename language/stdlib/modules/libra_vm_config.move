address 0x0:

module LibraVMConfig {
    use 0x0::LibraConfig;
    use 0x0::Transaction;

    // The struct to hold all config data needed to operate the LibraVM.
    // * publishing_option: Defines Scripts/Modules that are allowed to execute in the current configruation.
    // * gas_schedule: Cost of running the VM.
    struct T {
        publishing_option: vector<u8>,
        gas_schedule: Self::GasSchedule,
    }

    // The gas schedule keeps two separate schedules for the gas:
    // * The instruction_schedule: This holds the gas for each bytecode instruction.
    // * The native_schedule: This holds the gas for used (per-byte operated over) for each native
    //   function.
    // A couple notes:
    // 1. In the case that an instruction is deleted from the bytecode, that part of the cost schedule
    //    still needs to remain the same; once a slot in the table is taken by an instruction, that is its
    //    slot for the rest of time (since that instruction could already exist in a module on-chain).
    // 2. The initialization of the module will publish the instruction table to the association
    //    address, and will preload the vector with the gas schedule for instructions. The VM will then
    //    load this into memory at the startup of each block.
    struct GasSchedule {
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
        gas_constants: Self::GasConstants,
    }

    struct GasConstants {
        /// The cost per-byte written to global storage.
        global_memory_per_byte_cost: u64,

        /// The cost per-byte written to storage.
        global_memory_per_byte_write_cost: u64,

        /// We charge one unit of gas per-byte for the first 600 bytes
        min_transaction_gas_units: u64,

        /// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
        large_transaction_cutoff: u64,

        /// The units of gas that should be charged per byte for every transaction.
        instrinsic_gas_per_byte: u64,

        /// 1 nanosecond should equal one unit of computational gas. We bound the maximum
        /// computational time of any given transaction at 10 milliseconds. We want this number and
        /// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
        ///         MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
        maximum_number_of_gas_units: u64,

        /// The minimum gas price that a transaction can be submitted with.
        min_price_per_gas_unit: u64,

        /// The maximum gas unit price that a transaction can be submitted with.
        max_price_per_gas_unit: u64,

        max_transaction_size_in_bytes: u64,
    }

    // Initialize the table under the association account
    public fun initialize(
        publishing_option: vector<u8>,
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>
    ) {
        let gas_constants = GasConstants {
            global_memory_per_byte_cost: 8,
            global_memory_per_byte_write_cost: 8,
            min_transaction_gas_units: 600,
            large_transaction_cutoff: 600,
            instrinsic_gas_per_byte: 8,
            maximum_number_of_gas_units: 2000000,
            min_price_per_gas_unit: 0,
            max_price_per_gas_unit: 10000,
            max_transaction_size_in_bytes: 4096,
        };

        LibraConfig::publish_new_config<Self::T>(T {
            publishing_option,
            gas_schedule: GasSchedule {
                instruction_schedule,
                native_schedule,
                gas_constants,
            },
        })
    }

    public fun set_publishing_option(publishing_option: vector<u8>) {
        let current_config = LibraConfig::get<Self::T>(Transaction::sender());
        current_config.publishing_option = publishing_option;
        LibraConfig::set<Self::T>(Transaction::sender(), current_config )
    }
}

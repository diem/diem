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
    }

    // Initialize the table under the association account
    public fun initialize(
        publishing_option: vector<u8>,
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>
    ) {
        LibraConfig::publish_new_config<Self::T>(T {
            publishing_option,
            gas_schedule: GasSchedule {
                instruction_schedule,
                native_schedule,
            }
        })
    }

    public fun set_publishing_option(publishing_option: vector<u8>) {
        let current_config = LibraConfig::get<Self::T>(Transaction::sender());
        current_config.publishing_option = publishing_option;
        LibraConfig::set<Self::T>(Transaction::sender(), current_config )
    }
}

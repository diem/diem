address 0x0 {

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
module GasSchedule {
    use 0x0::Vector;
    use 0x0::Transaction;

    // The gas cost for each instruction is represented using two amounts;
    // one for the cpu, and the other for storage.
    struct Cost {
      cpu: u64,
      storage: u64,
    }

    resource struct T {
        instruction_schedule: vector<Cost>,
        native_schedule: vector<Cost>,
    }

    // Initialize the table under the association account
    fun initialize(gas_schedule: T) {
        Transaction::assert(Transaction::sender() == 0xA550C18, 0);
        move_to_sender<T>(gas_schedule);
    }

    public fun instruction_table_size(): u64 acquires T {
        let table = borrow_global<T>(0xA550C18);
        let instruction_table_len = Vector::length(&table.instruction_schedule);
        instruction_table_len
    }

    public fun native_table_size(): u64 acquires T {
        let table = borrow_global<T>(0xA550C18);
        let native_table_len = Vector::length(&table.native_schedule);
        native_table_len
    }
}
}

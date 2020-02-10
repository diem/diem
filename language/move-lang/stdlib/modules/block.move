address 0x0:

module Block {
    use 0x0::Transaction;

    resource struct T {
        // Height of the current block
        height: u64,
    }

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize() {
        // Only callable by the Association address
        Transaction::assert(Transaction::sender() == 0xA550C18, 1);
        move_to_sender(T { height: 0 });
    }

    // Set the metadata for the current block.
    // The runtime always runs this before executing the transactions in a block.
    // TODO: make this private, support other metadata
    public fun prologue(height: u64) acquires T {
        let height_ref = &mut borrow_global_mut<T>(0xA550C18).height;
        // ensure that height increases by 1
        Transaction::assert(height == *height_ref + 1, 99); // TODO: standardize this error code
        *height_ref = *height_ref + 1;
    }

    // Get the current block height
    public fun get_current_height(): u64 acquires T {
        borrow_global<T>(0xA550C18).height
    }

}

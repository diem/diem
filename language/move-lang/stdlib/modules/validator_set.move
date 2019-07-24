address 0x0:

module ValidatorSet {
    use 0x0.Transaction;
    use 0x0.Event;
    use 0x0.LibraAccount;
    use 0x0.ValidatorConfig;
    use 0x0.Vector;

    struct ValidatorInfo {
        addr: address,
        consensus_pubkey: bytearray,
        consensus_voting_power: u64,
        network_signing_pubkey: bytearray,
        network_identity_pubkey: bytearray,
    }

    struct ChangeEvent {
        new_validator_set: Vector.T<ValidatorInfo>,
    }

    resource struct T {
        validators: Vector.T<ValidatorInfo>,
        change_events: Event.Handle<ChangeEvent>,
    }

    // This can only be invoked by the special validator set address, and only a single time.
    // Currently, it is invoked in the genesis transaction.
    public initialize() {
        // Only callable by the validator set address
        Transaction.assert(Transaction.sender() == 0x1D8, 1);
        move_to_sender(T {
            validators: Vector.empty(),
            change_events: Event.new_event_handle(),
        })
    }

    // Return the size of the current validator set
    public size(): u64 acquires T {
        borrow_global<T>(0x1D8).validators.length()
    }

    // Return true if addr is a current validator
    public is_validator(addr: address): bool acquires T {
        let validators = &borrow_global<T>(0x1D8).validators;
        let size = validators.length();
        if (size == 0) return false;

        let i = 0;
        while (i < size) {
            if (validators.borrow(i).addr == addr) return true;
            i = i + 1;
        };
        false
    }

    // TODO: Decide on access control policy. For now, we ensure that this is only callable from the
    // genesis txn. Obviously, we'll need a different policy once we support reconfiguration.
    add_validator(addr: address) acquires T {
        // A prospective validator must have an account
        Transaction.assert(LibraAccount.exists(addr), 17);

        let config = &ValidatorConfig.config(addr);
        let info = ValidatorInfo {
            addr,
            consensus_pubkey: config.consensus_pubkey(),
            // TODO: check for LIT, compute voting power based on LIT + stake
            consensus_voting_power: 1,
            network_signing_pubkey: config.network_signing_pubkey(),
            network_identity_pubkey: config.network_identity_pubkey(),
        };
        borrow_global_mut<T>(0x1D8).validators.push_back(info);
    }

    // Trigger a reconfiguation the Libra system by:
    // (1) Computing a new validator set and storing it on chain
    // (2) Emitting an event containing new validator set, which will be passed to the executor
    reconfigure() acquires T {
        // TODO: for now, we just emit the current validator set. Eventually, we'll compute the new
        // validator set from a larger list of candidate validators sorted by stake.
        let validator_set = borrow_global_mut<T>(0x1D8);
        validator_set.change_events.emit_event(
            ChangeEvent { new_validator_set: *&validator_set.validators }
        )
    }

    // Get the address of the i'th validator.
    public get_ith_validator_address(i: u64): address acquires T {
        let validator_set = borrow_global<T>(0x1D8);
        Transaction.assert(i < validator_set.validators.length(), 3);
        validator_set.validators.borrow(i).addr
    }
}

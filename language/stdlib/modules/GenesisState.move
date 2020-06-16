address 0x1 {

/// This file implements a state variable to track the status of genesis.
/// It is in a separate leave module, because it will be used by lots of
/// other modules to check whether functions are being called during the
/// genesis process or not. Putting it in Genesis would introduce a ton
/// of dependencies, but using GenesisState will not.
module GenesisState {
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    /// A singleton resource that stores the state of the genesis process
    /// on genesis_address()
    resource struct GenesisState {
        genesis_complete: bool,
    }

    /// **TODO:** May want to move to CoreAddresses, or just use
    /// ASSOCIATION_ROOT_ADDRESS
    public fun genesis_address(): address {
        CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    }

    /// Abort if genesis has already begun, to protect against unverified
    /// code re-starting genesis.
    public fun begin_genesis(account: &signer) {
        assert(Signer::address_of(account) == genesis_address(), 1942);
        move_to(account, GenesisState { genesis_complete: false })
    }

    public fun end_genesis() acquires GenesisState {
        let gen_state = borrow_global_mut<GenesisState>(genesis_address());
        gen_state.genesis_complete = true;
    }

    public fun is_during_genesis(): bool acquires GenesisState {
        let gen_state = borrow_global_mut<GenesisState>(genesis_address());
        gen_state.genesis_complete == false
    }

    public fun is_after_genesis(): bool acquires GenesisState {
        let gen_state = borrow_global_mut<GenesisState>(genesis_address());
        gen_state.genesis_complete == true
    }

    //**************** Specifications ****************

    /// # GenesisState
    ///
    /// ## GenesisState state machine
    ///
    /// There is a state machine that tracks the genesis process.  There are three states,
    /// and it must proceed through them in strict sequences.
    /// **before_genesis:** Before start of genesis, there are no instances of the resource,
    ///    and, of course, none stored at the genesis_address.
    /// **during_genesis:** the resource is stored at genesis_address and the "genesis_complete"
    ///    field is "false".
    /// **after_genesis:** When genesis completes, the resource is stored at genesis_address
    ///     and the "genesis_complete" field is true.
    ///
    /// Correctness of initialization in many modules depends on initialization not changing
    /// the state after the first call, so it is important that no code be able to cause
    /// the system to enter the genesis states out-of-order.

    spec module {
        /// Helper functions
        ///
        /// TODO: Consider moving to CoreAddresses
        define spec_genesis_address(): address {
            0xA550C18
        }

        define spec_genesis_state(): GenesisState {
            global<GenesisState>(spec_genesis_address())
        }

        define spec_is_before_genesis(): bool {
            !exists<GenesisState>(spec_genesis_address())
        }

        define spec_is_during_genesis(): bool {
            exists<GenesisState>(spec_genesis_address())
            && !global<GenesisState>(spec_genesis_address()).genesis_complete
        }

        define spec_is_after_genesis(): bool {
            exists<GenesisState>(spec_genesis_address())
            && global<GenesisState>(spec_genesis_address()).genesis_complete
        }
    }

    spec schema TransBeginToDuring {
        /// **Informally:** If current state is before genesis, the very next state
        /// must be during genesis.
        /// Note that this does not allow the system to remain in "before genesis" for
        /// even one function call, so the transition must happen in the first function
        /// call.
        ensures old(spec_is_before_genesis()) ==> spec_is_during_genesis();
    }
    spec module {
        /// `genesis_address` is skipped because it does not change the state,
        /// which is specified and proved separately.
        apply TransBeginToDuring to * except genesis_address;
    }

    spec schema TransBeginToAfter {
        /// **Informally:** If the system has started but not finished genesis,
        /// the next state will either be the same, or it will end genesis.
        ensures old(spec_is_during_genesis())
                ==> (spec_is_during_genesis() || spec_is_after_genesis());
    }
    spec module {
        apply TransBeginToAfter to *;
    }

    spec schema AfterPersists {
        /// **Informally:** If the system has started but not finished genesis,
        /// the next state will either be the same, or it will end genesis.
        ensures old(spec_is_after_genesis()) ==> spec_is_after_genesis();
    }
    spec module {
        apply AfterPersists to *;
    }

    spec fun genesis_address {
        /// Does not change genesis state. This is because of "except genesis_address"
        /// in the apply of TransBeginToDuring.
        ensures old(spec_genesis_state()) == spec_genesis_state();
    }

    spec fun begin_genesis {
        ensures spec_is_during_genesis();
    }

    spec fun end_genesis {
         /// Requires that `end_genesis` only be called during genesis, which
         /// means that it can only be called once. There is no great harm if
         /// it is called multiple times, but reporting it as an error might
         /// catch code bugs.
        requires spec_is_during_genesis();
        ensures spec_is_after_genesis();
    }

}
}
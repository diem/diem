module TestEnsuresFalseSmokeTest {
    use 0x1::Signer;

    spec module {
        pragma verify = false;
        // pragma always_aborts_test = true;
        pragma const_field_test = true;
        pragma const_sc_addr = 0x0;
    }

    /// NOTE: Example motivated by !preburn.is_approved example in libra.move
    resource struct SomeResource {
        can_move: bool,
        never_moves: bool,
    }

    resource struct SomeResource2<T> {
        can_move: bool,
        never_moves: bool,
    }

    public fun init_some_resource2<T>(account: &signer) {
        move_to(account, SomeResource2<T> { can_move: false, never_moves: true });
    }

    public fun init_some_resource(account: &signer) {
        move_to(account, SomeResource { can_move: false, never_moves: true });
    }
    spec schema SomeResourceNeverMoves {
        invariant module forall addr: address where exists<SomeResource>(addr):
                            !global<SomeResource>(addr).can_move && 1 == 1;
    }
    spec module {
        apply SomeResourceNeverMoves to *;
    }
    spec fun init_some_resource {
        aborts_if exists<SomeResource>(Signer::spec_address_of(account));
        ensures exists<SomeResource>(Signer::spec_address_of(account));
        ensures !global<SomeResource>(Signer::spec_address_of(account)).can_move;
    }

    public fun remove_some_resource(account: &signer)
        acquires SomeResource
    {
        let can_remove = borrow_global<SomeResource>(Signer::address_of(account)).can_move;
        if (!can_remove) {
            abort 1
        };
    }
    spec fun remove_some_resource {
        // ensures false;
    }

    /// Example check that the variable never changes
    spec schema ConstantCanMove {
        // ensures forall addr: address:
        //             old(exists<SomeResource>(addr)) ==>
        //                 global<SomeResource>(addr).can_move == old(global<SomeResource>(addr).can_move);
        ensures old(exists<SomeResource>(0x0))
                    ==> global<SomeResource>(0x0).never_moves
                        == old(global<SomeResource>(0x0).never_moves);
    }
    spec module {
        apply ConstantCanMove to *;
    }

    // /// Function that changes the can_move constant
    public fun set_can_move(account: &signer)
        acquires SomeResource
    {
        let can_move = &mut borrow_global_mut<SomeResource>(Signer::address_of(account)).can_move;
        *can_move = true;
    }

    public fun set_can_move2<T>(account: &signer)
        acquires SomeResource2
    {
        let can_move = &mut borrow_global_mut<SomeResource2<T>>(Signer::address_of(account)).can_move;
        *can_move = true;
    }

    public fun set_never_move(account: &signer)
        acquires SomeResource
    {
        let never_moves = &mut borrow_global_mut<SomeResource>(Signer::address_of(account)).never_moves;
        *never_moves = false;
    }
}

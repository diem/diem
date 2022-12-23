address 0x2 {
module S {
    struct Storage<X: store, Y: store> has key {
        x: X,
        y: Y,
        v: u8,
    }

    // F1: <concrete, concrete>
	public fun publish_u64_bool(account: signer, x: u64, y: bool) {
	    move_to(&account, Storage { x, y, v: 0 })
	}

    // F2: <concrete, generic>
	public fun publish_u64_y<Y: store>(account: signer, x: u64, y: Y) {
	    move_to(&account, Storage { x, y, v: 1 })
	}

    // F3: <generic, concrete>
	public fun publish_x_bool<X: store>(account: signer, x: X, y: bool) {
	    move_to(&account, Storage { x, y, v: 2 })
	}

    // F4: <generic, generic>
	public fun publish_x_y<X: store, Y: store>(account: signer, x: X, y: Y) {
	    move_to(&account, Storage { x, y, v: 3 })
	}
}

module A {
    use 0x2::S;

    // I1: <concrete, concrete>
    invariant
        exists<S::Storage<u64, bool>>(@0x22)
            ==> global<S::Storage<u64, bool>>(@0x22).x == 1;

    // I2: <concrete, generic>
    invariant<Y>
        exists<S::Storage<u64, Y>>(@0x23)
            ==> global<S::Storage<u64, Y>>(@0x23).x > 0;

    // I3: <generic, concrete>
    invariant<X>
        exists<S::Storage<X, bool>>(@0x24)
            ==> global<S::Storage<X, bool>>(@0x24).y;

    // I4: <generic, generic>
    invariant<X, Y>
        (exists<S::Storage<X, Y>>(@0x25) && exists<S::Storage<X, Y>>(@0x26))
            ==> global<S::Storage<X, Y>>(@0x25) == global<S::Storage<X, Y>>(@0x26);

    public fun good(account1: signer, account2: signer) {
        S::publish_x_y<u64, bool>(account1, 1, true);
        S::publish_x_y<u64, bool>(account2, 1, true);
    }

    // all invariants (I1-I4) should be disproved in all functions (F1-F4)
}
}

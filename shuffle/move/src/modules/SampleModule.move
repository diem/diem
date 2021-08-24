module 0x1::SampleModule {
    struct Coin has key, store {
        value: u64,
    }

    const COIN_ALREADY_EXISTS: u64 = 0;
    const COIN_DOES_NOT_EXIST: u64 = 1;
    const COIN_HAS_WRONG_VALUE: u64 = 2;

    public(script) fun mint_coin(owner: signer, amount: u64) {
        move_to(&owner, Coin { value: amount } );
    }

    #[test(owner = @0xA)]
    public(script) fun test_mint_coin(owner: signer) acquires Coin {
        // Before publishing, there is no `Coin` resource under the address.
        assert(!exists<Coin>(@0xA), COIN_ALREADY_EXISTS);

        mint_coin(owner, 42);

        // Check that there is a `Coin` resource published with the correct value.
        assert(exists<Coin>(@0xA), COIN_DOES_NOT_EXIST);
        assert(borrow_global<Coin>(@0xA).value == 42, COIN_HAS_WRONG_VALUE);
    }
}

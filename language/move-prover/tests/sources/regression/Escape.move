address 0x1 {

module Escape {
    use 0x1::Signer;

    struct IndoorThing has key, store { }

    struct OutdoorThing has key, store { }

    struct Wrapper<Thing: key + store> has key { thing: Thing }

    public fun initialize(account: &signer) {
        let owner = Signer::address_of(account);
        assert(owner == @0x123, 0);
        move_to<Wrapper<IndoorThing>>(account, Wrapper{ thing: IndoorThing {} });
        move_to<Wrapper<OutdoorThing>>(account, Wrapper { thing: OutdoorThing {}});
    }

   public fun new_outdoor_thing(): OutdoorThing {
        OutdoorThing { }
    }

    /// Calling module can install whatever object.
    public fun install<Thing: key + store>(account: &signer, thing: Thing) {
        move_to<Wrapper<Thing>>(account, Wrapper{ thing });
    }

    // TODO: Both do not verify, but the first could be verified because an
    // `IndoorThing` can never escape from the current module (while an
    // `OutdoorThing` can because of the `new_outdoor_thing` function.
    invariant forall addr: address where exists<Wrapper<IndoorThing>>(addr): addr == @0x123;
    invariant forall addr: address where exists<Wrapper<OutdoorThing>>(addr): addr == @0x123;
}

}

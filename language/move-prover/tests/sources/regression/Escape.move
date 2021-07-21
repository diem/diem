address 0x1 {

module Escape {
    use Std::Signer;

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

    // TODO: Currently both invariants do not verify, which is expected.
    //
    // But the first invariant could be verified because an `IndoorThing` can
    // never escape from the current modul. By escape, we mean, holding an
    // object of type `IndoorThing`. This is a fact given by the type system.
    //
    // In comparision, an `OutdoorThing` can escape because that the
    // `new_outdoor_thing` function returns an object of the `OutdoorThing` type
    invariant forall addr: address where exists<Wrapper<IndoorThing>>(addr): addr == @0x123;
    invariant forall addr: address where exists<Wrapper<OutdoorThing>>(addr): addr == @0x123;
}
}

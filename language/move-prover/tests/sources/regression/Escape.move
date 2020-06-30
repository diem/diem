address 0x1 {

module Escape {
    use 0x1::Signer;

    resource struct IndoorThing { }

    resource struct OutdoorThing { }

    resource struct Wrapper<Thing: resource> { thing: Thing }

    public fun initialize(account: &signer) {
        let owner = Signer::address_of(account);
        assert(owner == 0x123, 0);
        move_to<Wrapper<IndoorThing>>(account, Wrapper{ thing: IndoorThing {} });
        move_to<Wrapper<OutdoorThing>>(account, Wrapper { thing: OutdoorThing {}});
    }

   public fun new_outdoor_thing(): OutdoorThing {
        OutdoorThing { }
    }

    /// Calling module can install whatever object
    public fun install<Thing: resource>(account: &signer, thing: Wrapper<Thing>) {
        move_to<Wrapper<Thing>>(account, thing);
    }

// **************** Specifications ****************


    /// TODO BUG (dd): When the next is commented in, the Prover reports an error.  Another module
    /// or transaction could call `new_outdoor_thing` to obtain an OutdoorThing instance, then call
    /// `install(account_456, od_thing)` to store at a the address of signer account_456 (e.g., 0x456),
    /// violating the condition.
    // spec schema OutdoorThingUnique {
    //     invariant module forall addr: address where exists<Wrapper<OutdoorThing>>(addr):  addr == 0x123;
    // }

    // spec module {
    //     apply OutdoorThingUnique to *<T>, *;
    // }

    /// TODO BUG (dd): This reports a false error for the same reason as the previous invariant (the
    /// error is not reported when the previous invariant generates an error, which is why that invariant
    /// is commented out). The prover does the same thing in both cases, but this condition actually
    /// holds because there is no way for another module to get an instance of an IndoorThing with which
    /// to call `install`.  The prover needs some additional bookkeeping to note whether an instance of a
    /// type can escape in order to avoid this false error.
    spec schema IndoorThingUnique {
        invariant module forall addr: address where exists<Wrapper<IndoorThing>>(addr): addr == 0x123;
    }

    spec module {
        apply IndoorThingUnique to *<T>, *;
    }


}
}

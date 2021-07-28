module Std::Signer {
    // Borrows the address of the signer
    // Conceptually, you can think of the `signer` as being a struct wrapper arround an
    // address
    // ```
    // struct Signer has drop { addr: address }
    // ```
    // `borrow_address` borrows this inner field
    native public fun borrow_address(s: &signer): &address;

    // Copies the address of the signer
    public fun address_of(s: &signer): address {
        *borrow_address(s)
    }
    spec address_of {
        pragma opaque;
        aborts_if false;
        ensures result == spec_address_of(s);
    }

    /// Specification version of `Self::address_of`.
    spec native fun spec_address_of(account: signer): address;

    /// Return true only if `a` is a transaction signer. This is a spec function only available in spec.
    spec fun is_signer(addr: address): bool;

    spec module {
        axiom forall s:signer: is_signer(spec_address_of(s));
    }
}

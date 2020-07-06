address 0x1 {
module Signer {
    // Borrows the address of the signer
    // Conceptually, you can think of the `signer` as being a resource struct wrapper arround an
    // address
    // ```
    // resource struct Signer { addr: address }
    // ```
    // `borrow_address` borrows this inner field
    native public fun borrow_address(s: &signer): &address;

    // Copies the address of the signer
    public fun address_of(s: &signer): address {
        *borrow_address(s)
    }
    spec fun address_of {
        pragma opaque = true;
        aborts_if false;
        ensures result == spec_address_of(s);
    }

    spec module {
        /// Specification version of `Self::address_of`.
        native define spec_address_of(account: signer): address;
    }
}
}

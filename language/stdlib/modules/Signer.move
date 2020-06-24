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

    spec module {
        /// Helper function that returns the address of the signer.
        native define get_address(account: signer): address;
    }

    spec fun address_of {
        aborts_if false;
        ensures result == get_address(s);
    }

}
}

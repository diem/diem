address 0x0 {
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
}
}

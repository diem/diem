address 0x0:

module Transaction {
    native public gas_unit_price(): u64;
    native public max_gas_units(): u64;
    native public gas_remaining(): u64;
    native public sender(): address;
    native public sequence_number(): u64;
    native public public_key(): bytearray;

    // inlined
    public assert(check: bool, code: u64) {
        if (check) () else abort code
    }
}

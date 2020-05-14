address 0x0 {

module Transaction {
    native public fun gas_unit_price(): u64;
    native public fun max_gas_units(): u64;
    native public fun gas_remaining(): u64;
    native public fun sender(): address;
    native public fun sequence_number(): u64;
    native public fun public_key(): vector<u8>;

    // inlined
    public fun assert(check: bool, code: u64) {
        if (check) () else abort code
    }
    spec fun assert {
        aborts_if !check;
    }
}

}

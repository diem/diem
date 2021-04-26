module 0x8675309::M {
    struct R has key { data: vector<u8> }

    fun is_ok_(_addr: &address, _data: &vector<u8>): bool {
        true
    }

    public fun is_ok(addr: address): bool acquires R {
        is_ok_(&addr, &borrow_global<R>(@0x1D8).data)
    }

    // ImmBorrowLoc(0),
    // LdAddr(1),
    // ImmBorrowGlobal(StructDefinitionIndex(4)),
    // ImmBorrowField(FieldHandleIndex(0)),
    // Call(45),
    // Ret,
}

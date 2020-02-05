module Tester {
    struct X { f: u64 }

    fun bump_and_give(x_ref: &mut X): &u64 {
        x_ref.f = x_ref.f + 1;
        &x_ref.f
    }

    fun contrived_example_no(x_ref: &mut X): &u64 {
        let returned_ref = bump_and_give(x_ref);
        // ERROR Cannot mutably borrow from `x_ref` it is being borrowed by `returned_ref`
        0x0::Transaction::assert(*returned_ref == *freeze(&mut x_ref.f) + 1, 42);
        returned_ref
    }

    fun contrived_example_valid(x_ref: &mut X): &u64 {
        let returned_ref = bump_and_give(x_ref);
        // This is still valid, but might not be in other cases. See other Imm Borrow tests
        // i.e. you might hit FreezeRefExistsMutableBorrowError
        0x0::Transaction::assert(*returned_ref == *(&freeze(x_ref).f) + 1, 42);
        returned_ref
    }
}

// check: BORROWFIELD_EXISTS_MUTABLE_BORROW_ERROR

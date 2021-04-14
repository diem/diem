address 0x1 {
module B {
    #[test_only]
    struct TestingStruct has drop { x: u64 }

    #[test_only]
    public fun construct_with_number(x: u64): TestingStruct {
        TestingStruct { x }
    }

    #[test_only]
    public fun get_struct_x_field(s: &TestingStruct): u64 {
        s.x
    }
}

module M {
    #[test_only]
    use 0x1::B;

    #[test]
    fun make_sure_number_matches() {
        let s = B::construct_with_number(0);
        assert(B::get_struct_x_field(&s) == 0, 0);
    }

    #[test, expected_failure]
    fun make_sure_not_other_number() {
        let s = B::construct_with_number(0);
        assert(B::get_struct_x_field(&s) != 0, 0);
    }
}
}

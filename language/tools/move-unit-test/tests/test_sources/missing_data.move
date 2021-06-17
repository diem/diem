module 0x1::MissingData {
    struct Missing has key { }

    #[test]
    fun missing_data() acquires Missing {
        borrow_global<Missing>(@0x0);
    }
}

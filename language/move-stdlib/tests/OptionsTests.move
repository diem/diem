#[test_only]
module Std::OptionTests {
    use Std::Option;


    #[test]
    fun option_none_is_none() {
        let none = Option::none<u64>();
        assert(Option::is_none(&none), 0);
        assert(!Option::is_some(&none), 1);
    }

    #[test]
    fun option_some_is_some() {
        let some = Option::some(5);
        assert(!Option::is_none(&some), 0);
        assert(Option::is_some(&some), 1);
    }

    #[test]
    fun option_contains() {
        let none = Option::none<u64>();
        let some = Option::some(5);
        let some_other = Option::some(6);
        assert(Option::contains(&some, &5), 0);
        assert(Option::contains(&some_other, &6), 1);
        assert(!Option::contains(&none, &5), 2);
        assert(!Option::contains(&some_other, &5), 3);
    }

    #[test]
    fun option_borrow_some() {
        let some = Option::some(5);
        let some_other = Option::some(6);
        assert(*Option::borrow(&some) == 5, 3);
        assert(*Option::borrow(&some_other) == 6, 4);
    }

    #[test]
    #[expected_failure(abort_code = 263)]
    fun option_borrow_none() {
        Option::borrow(&Option::none<u64>());
    }

    #[test]
    fun borrow_mut_some() {
        let some = Option::some(1);
        let ref = Option::borrow_mut(&mut some);
        *ref = 10;
        assert(*Option::borrow(&some) == 10, 0);
    }

    #[test]
    #[expected_failure(abort_code = 263)]
    fun borrow_mut_none() {
        Option::borrow_mut(&mut Option::none<u64>());
    }

    #[test]
    fun borrow_with_default() {
        let none = Option::none<u64>();
        let some = Option::some(5);
        assert(*Option::borrow_with_default(&some, &7) == 5, 0);
        assert(*Option::borrow_with_default(&none, &7) == 7, 1);
    }

    #[test]
    fun get_with_default() {
        let none = Option::none<u64>();
        let some = Option::some(5);
        assert(Option::get_with_default(&some, 7) == 5, 0);
        assert(Option::get_with_default(&none, 7) == 7, 1);
    }

    #[test]
    fun extract_some() {
        let opt = Option::some(1);
        assert(Option::extract(&mut opt) == 1, 0);
        assert(Option::is_none(&opt), 1);
    }

    #[test]
    #[expected_failure(abort_code = 263)]
    fun extract_none() {
        Option::extract(&mut Option::none<u64>());
    }

    #[test]
    fun swap_some() {
        let some = Option::some(5);
        assert(Option::swap(&mut some, 1) == 5, 0);
        assert(*Option::borrow(&some) == 1, 1);
    }

    #[test]
    #[expected_failure(abort_code = 263)]
    fun swap_none() {
        Option::swap(&mut Option::none<u64>(), 1);
    }

    #[test]
    fun fill_none() {
        let none = Option::none<u64>();
        Option::fill(&mut none, 3);
        assert(Option::is_some(&none), 0);
        assert(*Option::borrow(&none) == 3, 1);
    }

    #[test]
    #[expected_failure(abort_code = 7)]
    fun fill_some() {
        Option::fill(&mut Option::some(3), 0);
    }

    #[test]
    fun destroy_with_default() {
        assert(Option::destroy_with_default(Option::none<u64>(), 4) == 4, 0);
        assert(Option::destroy_with_default(Option::some(4), 5) == 4, 1);
    }

    #[test]
    fun destroy_some() {
        assert(Option::destroy_some(Option::some(4)) == 4, 0);
    }

    #[test]
    #[expected_failure(abort_code = 263)]
    fun destroy_some_none() {
        Option::destroy_some(Option::none<u64>());
    }

    #[test]
    fun destroy_none() {
        Option::destroy_none(Option::none<u64>());
    }

    #[test]
    #[expected_failure(abort_code = 7)]
    fun destroy_none_some() {
        Option::destroy_none(Option::some<u64>(0));
    }
}

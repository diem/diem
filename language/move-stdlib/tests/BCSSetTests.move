#[test_only]
module Std::BCSSetTests {
    use Std::BCSSet::Self;
    use Std::Option;

    #[test]
    fun singleton_set() {
        // test borrow, borrow_index, is_mem, insert on simplest case
        let s = BCSSet::empty<u64>();
        assert(BCSSet::size(&s) == 0, 0);
        BCSSet::insert(&mut s, 7);
        assert(*BCSSet::borrow(&s, 0) == 7, 1);
        assert(BCSSet::is_mem(&s, &7), 2);
    }

    #[test]
    fun check_order_after_insert() {
        // add 10 elements in arbitrary order
        let s = BCSSet::empty<u64>();
        BCSSet::insert(&mut s, 4);
        BCSSet::insert(&mut s, 6);
        BCSSet::insert(&mut s, 1);
        BCSSet::insert(&mut s, 8);
        BCSSet::insert(&mut s, 3);
        BCSSet::insert(&mut s, 7);
        BCSSet::insert(&mut s, 9);
        BCSSet::insert(&mut s, 0);
        BCSSet::insert(&mut s, 2);
        BCSSet::insert(&mut s, 5);
        assert(BCSSet::size(&s) == 10, 0);

        // check membership, sortedness, insertion failure
        let i = 0;
        let len = BCSSet::size(&s);
        while (i < len) {
            assert(*BCSSet::borrow(&s, i) == i, 1);
            assert(Option::destroy_some(BCSSet::borrow_index(&s, &i)) == i, 2);
            assert(BCSSet::is_mem(&s, &i), 3);
            assert(BCSSet::insert(&mut s, i) == false, 4);
            assert(Option::destroy_some(BCSSet::insert_or_return(&mut s, i)) == i, 5);
            i = i + 1
        };
        assert(BCSSet::size(&s) == len, 5);
    }
}

// Tests for the non-aborting behavior of Option functions
script {
use 0x1::Option;

fun main() {
    // constructors for some/none + boolean is functions
    let none = Option::none<u64>();
    assert(Option::is_none(&none), 8001);
    assert(!Option::is_some(&none), 8002);

    let some = Option::some(5);
    assert(!Option::is_none(&some), 8003);
    assert(Option::is_some(&some), 8004);

    // contains/immutable borrowing
    assert(Option::contains(&some, &5), 8005);
    assert(!Option::contains(&none, &5), 8077);
    assert(*Option::borrow(&some) == 5, 8006);

    // borrow/get_with_default
    assert(*Option::borrow_with_default(&some, &7) == 5, 8007);
    assert(*Option::borrow_with_default(&none, &7) == 7, 8008);
    assert(Option::get_with_default(&some, 7) == 5, 8009);
    assert(Option::get_with_default(&none, 7) == 7, 8010);

    // mutation: swap, extract
    Option::swap(&mut some, 1);
    assert(*Option::borrow(&some) == 1, 8011);
    assert(Option::extract(&mut some) == 1, 8012);
    let now_empty = some;
    assert(Option::is_none(&now_empty), 8013);
    assert(&now_empty == &none, 8014);

    // fill, borrow_mut
    Option::fill(&mut none, 3);
    let three = none;
    assert(Option::is_some(&three), 8015);
    assert(*Option::borrow(&three) == 3, 8016);
    let three_ref = Option::borrow_mut(&mut three);
    *three_ref = 10;
    let ten = three;
    assert(*Option::borrow(&ten) == 10, 8017);

    // destroy_with_default, destroy_some, destroy_none
    assert(Option::destroy_with_default(Option::none<u64>(), 4) == 4, 8018);
    assert(Option::destroy_with_default(Option::some(4), 5) == 4, 8019);
    assert(Option::destroy_some(Option::some(4)) == 4, 8020);
    Option::destroy_none(Option::none<u64>());

    // letting an Option<u64> go out of scope is also ok
    let _ = Option::some(7);
}
}

// check: EXECUTED

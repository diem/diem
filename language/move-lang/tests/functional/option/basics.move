// Tests for the non-aborting behavior of Option functions
script {
use 0x0::Option;
use 0x0::Transaction;

fun main() {
    // constructors for some/none + boolean is functions
    let none = Option::none<u64>();
    Transaction::assert(Option::is_none(&none), 8001);
    Transaction::assert(!Option::is_some(&none), 8002);

    let some = Option::some(5);
    Transaction::assert(!Option::is_none(&some), 8003);
    Transaction::assert(Option::is_some(&some), 8004);

    // contains/immutable borrowing
    Transaction::assert(Option::contains(&some, &5), 8005);
    Transaction::assert(!Option::contains(&none, &5), 8077);
    Transaction::assert(*Option::borrow(&some) == 5, 8006);

    // borrow/get_with_default
    Transaction::assert(*Option::borrow_with_default(&some, &7) == 5, 8007);
    Transaction::assert(*Option::borrow_with_default(&none, &7) == 7, 8008);
    Transaction::assert(Option::get_with_default(&some, 7) == 5, 8009);
    Transaction::assert(Option::get_with_default(&none, 7) == 7, 8010);

    // mutation: swap, extract
    Option::swap(&mut some, 1);
    Transaction::assert(*Option::borrow(&some) == 1, 8011);
    Transaction::assert(Option::extract(&mut some) == 1, 8012);
    let now_empty = some;
    Transaction::assert(Option::is_none(&now_empty), 8013);
    Transaction::assert(&now_empty == &none, 8014);

    // fill, borrow_mut
    Option::fill(&mut none, 3);
    let three = none;
    Transaction::assert(Option::is_some(&three), 8015);
    Transaction::assert(*Option::borrow(&three) == 3, 8016);
    let three_ref = Option::borrow_mut(&mut three);
    *three_ref = 10;
    let ten = three;
    Transaction::assert(*Option::borrow(&ten) == 10, 8017);

    // destroy_with_default, destroy_some, destroy_none
    Transaction::assert(Option::destroy_with_default(Option::none<u64>(), 4) == 4, 8018);
    Transaction::assert(Option::destroy_some(Option::some(4)) == 4, 8019);
    Option::destroy_none(Option::none<u64>());

    // letting an Option<u64> go out of scope is also ok
    let _ = Option::some(7);
}
}

// check: EXECUTED

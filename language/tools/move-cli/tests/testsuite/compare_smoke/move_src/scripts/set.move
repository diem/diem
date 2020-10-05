script {
use 0x2::Set;
fun main() {
    // add 10 elements in arbitrary order, check sortedness at the end
    let s = Set::empty<u64>();
    Set::insert(&mut s, 4);
    Set::insert(&mut s, 6);
    Set::insert(&mut s, 1);
    Set::insert(&mut s, 8);
    Set::insert(&mut s, 3);
    Set::insert(&mut s, 7);
    Set::insert(&mut s, 9);
    Set::insert(&mut s, 0);
    Set::insert(&mut s, 2);
    Set::insert(&mut s, 5);
    assert(Set::size(&s) == 10, 70002);

    let i = 0;
    while (i < Set::size(&s)) {
        assert(*Set::borrow(&s, i) == i, 70003);
        i = i + 1
    };

    // simple singleton case
    let s = Set::empty<u64>();
    Set::insert(&mut s, 7);
    assert(*Set::borrow(&s, 0) == 7, 7000);
    assert(Set::is_mem(&s, &7), 7001);

    Set::insert(&mut s, 7) // will abort with 999
}
}

// Generic set that leverages Compare::cmp.
// This is a reasonable smoke test for the Compare module, but don't actually use this without
// singificantly more testing/thought about the API!
module Set {
    use 0x1::Compare;
    use 0x1::LCS;
    use 0x1::Vector;

    struct T<Elem> { v: vector<Elem> }

    public fun empty<Elem>(): T<Elem> {
        T { v: Vector::empty() }
    }

    public fun size<Elem>(t: &T<Elem>): u64 {
       Vector::length(&t.v)
    }

    public fun borrow<Elem>(t: &T<Elem>, index: u64): &Elem {
        Vector::borrow(&t.v, index)
    }

    fun find<Elem>(t: &T<Elem>, e: &Elem): (u64, bool) {
        let e_lcs = LCS::to_bytes(e);
        let v = &t.v;
        // use binary search to locate `e` (if it exists)
        let left = 0;
        let len =  Vector::length(v);
        if (len == 0) {
            return (0, false)
        };
        let right = len - 1;
        while (left <= right) {
            let mid = (left + right) / 2;
            let cmp = Compare::cmp_lcs_bytes(&LCS::to_bytes(Vector::borrow(v, mid)), &e_lcs);
            if (cmp == 0u8) {
                return (mid, true)
            } else if (cmp == 1u8) {
                left = mid + 1
            } else { // cmp == 2u8
                if (mid == 0) {
                    return (0, false)
                };
                assert(mid != 0, 88);
                right = mid -1
            }
        };

        (left, false)
    }

    // return true if `e` is already present in `t`, abort otherwise
    public fun insert<Elem>(t: &mut T<Elem>, e: Elem) {
        let (insert_at, found) = find(t, &e);
        if (found) {
            abort(999)
        };
        let v = &mut t.v;
        // TODO: Vector::insert(index, e) would be useful here.
        let i = Vector::length(v);
        // add e to the end and then move it  to the left until we hit `insert_at`
        Vector::push_back(v, e);
        while (i > insert_at) {
            Vector::swap(v, i, i - 1);
            i = i - 1;
        }
    }

    public fun is_mem<Elem>(t: &T<Elem>, e: &Elem): bool {
        let (_index, found) = find(t, e);
        found
    }

}

//! new-transaction
script {
use {{default}}::Set;
fun main() {
    // simple singleton case
    let s = Set::empty<u64>();
    Set::insert(&mut s, 7);
    assert(*Set::borrow(&s, 0) == 7, 7000);
    assert(Set::is_mem(&s, &7), 7001);

    Set::insert(&mut s, 7) // will abort with 999
}
}

// check: "Keep(ABORTED { code: 999,"

//! new-transaction
//! gas-price: 0
script {
use {{default}}::Set;
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
    }
}
}
// check: "Keep(EXECUTED)"

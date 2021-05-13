// Generic set that leverages Compare::cmp.
// This is a reasonable smoke test for the Compare module, but don't actually use this without
// singificantly more testing/thought about the API!
address 0x2 {
module Set {
    use Std::Compare;
    use Std::BCS;
    use Std::Vector;

    struct T<Elem> has copy, drop, store { v: vector<Elem> }

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
        let e_bcs = BCS::to_bytes(e);
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
            let cmp = Compare::cmp_bcs_bytes(&BCS::to_bytes(Vector::borrow(v, mid)), &e_bcs);
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
}

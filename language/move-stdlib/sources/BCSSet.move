module Std::BCSSet {
    use Std::BCS;
    use Std::Option::{Self, Option};
    use Std::Vector;

    /// A set of elements ordered by their BCS serialization.
    /// Note that this is often different than the natural ordering on elements
    /// (e.g., for addresses)--see Std::Compare for a detailed explanation
    struct Set<Elem> has copy, drop, store {
        elements: vector<Elem>
    }

    /// Create an empty set
    public fun empty<Elem>(): Set<Elem> {
        Set { elements: Vector::empty() }
    }

    /// Add `e` to `s`. Return `e` wrapped in an `Option` if it is already
    /// present in `t`and `None` otherwise
    public fun insert_or_return<Elem>(s: &mut Set<Elem>, e: Elem): Option<Elem> {
        let (insert_at, found) = find(s, &e);
        if (found) {
            return Option::some(e)
        };
        // else, not found--insert
        let elements = &mut s.elements;
        // TODO: Vector::insert(index, e) would be useful here.
        let i = Vector::length(elements);
        // add e to the end and then move it  to the left until we hit `insert_at`
        Vector::push_back(elements, e);
        while (i > insert_at) {
            Vector::swap(elements, i, i - 1);
            i = i - 1;
        };
        Option::none()
    }

    /// Add `e` to `s` Return false if `e` was already present in `t` and `true` otherwise
    public fun insert<Elem: drop>(s: &mut Set<Elem>, e: Elem): bool {
        let e_opt = insert_or_return(s, e);
        Option::is_none(&e_opt)
    }

    /// Return true if `t` contains `e`, false otherwise
    public fun is_mem<Elem>(s: &Set<Elem>, e: &Elem): bool {
        let (_index, found) = find(s, e);
        found
    }

    /// If `t` contains `e`, return`e's` index wrapped in an `Option`.
    /// Otherwise, return `None`
    public fun borrow_index<Elem>(s: &Set<Elem>, e: &Elem): Option<u64> {
        let (index, found) = find(s, e);
        if (found) {
            Option::some(index)
        } else {
            Option::none()
        }
    }

    /// Borrow the `index`th element of `s`
    public fun borrow<Elem>(s: &Set<Elem>, index: u64): &Elem {
        Vector::borrow(&s.elements, index)
    }

    /// Return the cardinality of `s`
    public fun size<Elem>(s: &Set<Elem>): u64 {
       Vector::length(&s.elements)
    }

    /// Return the backing array of this Set
    public fun elements<Elem>(s: &Set<Elem>): &vector<Elem> {
        &s.elements
    }

    // workhorse of the module--find the position of an element using binary search
    // return its index and true if it exists, or the length of the vector and false
    // if it doesn't
    fun find<Elem>(s: &Set<Elem>, e: &Elem): (u64, bool) {
        let e_bcs = BCS::to_bytes(e);
        let elements = &s.elements;

        let left = 0;
        let len =  Vector::length(elements);
        if (len == 0) {
            return (0, false)
        };
        // use binary search to locate `e` (if it exists)
        let right = len - 1;
        while (left <= right) {
            let mid = (left + right) / 2;
            let cmp = BCS::cmp_bytes(&BCS::to_bytes(Vector::borrow(elements, mid)), &e_bcs);
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
}

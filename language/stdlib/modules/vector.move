address 0x0:

// A variable-sized container that can hold both unrestricted types and resources.
module Vector {
    native public fun empty<Element>(): vector<Element>;

    // Return the length of the vector.
    native public fun length<Element>(v: &vector<Element>): u64;

    // Acquire an immutable reference to the ith element of the vector.
    native public fun borrow<Element>(v: &vector<Element>, i: u64): &Element;

    // Add an element to the end of the vector.
    native public fun push_back<Element>(v: &mut vector<Element>, e: Element);

    // Get mutable reference to the ith element in the vector, abort if out of bound.
    native public fun borrow_mut<Element>(v: &mut vector<Element>, idx: u64): &mut Element;

    // Pop an element from the end of vector, abort if the vector is empty.
    native public fun pop_back<Element>(v: &mut vector<Element>): Element;

    // Destroy the vector, abort if not empty.
    native public fun destroy_empty<Element>(v: vector<Element>);

    // Swaps the elements at the i'th and j'th indices in the vector.
    native public fun swap<Element>(v: &mut vector<Element>, i: u64, j: u64);

    // Reverses the order of the elements in the vector in place.
    public fun reverse<Element>(v: &mut vector<Element>) {
        let len = length(v);
        if (len == 0) return ();

        let front_index = 0;
        let back_index = len -1;
        while (front_index < back_index) {
            swap(v, front_index, back_index);
            front_index = front_index + 1;
            back_index = back_index - 1;
        }
    }

    // Moves all of the elements of the `other` vector into the `lhs` vector.
    public fun append<Element>(lhs: &mut vector<Element>, other: vector<Element>) {
        reverse(&mut other);
        while (!is_empty(&other)) push_back(lhs, pop_back(&mut other));
        destroy_empty(other);
    }

    // Return true if the vector has no elements
    public fun is_empty<Element>(v: &vector<Element>): bool {
        length(v) == 0
    }

    // Return true if `e` is in the vector `v`
    public fun contains<Element>(v: &vector<Element>, e: &Element): bool {
        let i = 0;
        let len = length(v);
        while (i < len) {
            if (borrow(v, i) == e) return true;
            i = i + 1;
        };
        false
    }

    // Remove the `i`th element E of the vector, shifting all subsequent elements
    // It is O(n) and preserves ordering
    public fun remove<Element>(v: &mut vector<Element>, i: u64): Element {
        let len = length(v);
        // i out of bounds; abort
        if (i >= len) abort 10;

        len = len - 1;
        while (i < len) swap(v, i, { i = i + 1; i });
        pop_back(v)
    }

    // Remove the `i`th element E of the vector by swapping it with the last element,
    // and then popping it off
    // It is O(1), but does not preserve ordering
    public fun swap_remove<Element>(v: &mut vector<Element>, i: u64): Element {
        let last_idx = length(v) - 1;
        swap(v, i, last_idx);
        pop_back(v)
    }

}

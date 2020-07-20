// This file is created to verify the vector module in the standard library.
// This file is basically a clone of `stdlib/modules/vector.move` with renaming the module and function names.
// In this file, the functions with prefix of `verify_model` are verifying the corresponding built-in Boogie
// procedures that they inline (e.g., `verify_model_remove`).
// This file also attempts to verify the actual Move implementations of non-native functions (e.g., `verify_remove`),
// but it fails if loops are involved.

module VerifyVector {
    use 0x1::Vector;

    spec module {
        pragma verify = true;
    }

    fun verify_model_empty<Element>() : vector<Element> {
        Vector::empty<Element>() // inlining the built-in Boogie procedure
    }
    spec fun verify_model_empty {
        ensures len(result) == 0;
    }

    // Return the length of the vector.
    fun verify_model_length<Element>(v: &vector<Element>): u64 {
        Vector::length(v) // inlining the built-in Boogie procedure
    }
    spec fun verify_model_length {
        ensures result == len(v);
    }

    // Acquire an immutable reference to the ith element of the vector.
    fun verify_model_borrow<Element>(v: &vector<Element>, i: u64): &Element {
        Vector::borrow(v, i) // inlining the built-in Boogie procedure
    }
    spec fun verify_model_borrow {
        aborts_if i >= len(v);
        ensures result == v[i]; // TODO: enough?
    }

    // Add an element to the end of the vector.
    fun verify_model_push_back<Element>(v: &mut vector<Element>, e: Element) {
        Vector::push_back(v, e); // inlining the built-in Boogie procedure
    }
    spec fun verify_model_push_back {
        ensures len(v) == len(old(v)) + 1;
        ensures v[len(v)-1] == e;
        ensures old(v) == v[0..len(v)-1];
    }

    // Get mutable reference to the ith element in the vector, abort if out of bound.
    fun verify_model_borrow_mut<Element>(v: &mut vector<Element>, i: u64): &mut Element {
        Vector::borrow_mut(v, i) // inlining the built-in Boogie procedure
    }
    spec fun verify_model_borrow_mut {
        aborts_if i >= len(v);
        ensures result == v[i]; // TODO: enough?
    }

    // Pop an element from the end of vector, abort if the vector is empty.
    fun verify_model_pop_back<Element>(v: &mut vector<Element>): Element {
        Vector::pop_back(v) // inlining the built-in Boogie procedure
    }
    spec fun verify_model_pop_back {
        aborts_if len(v) == 0;
        ensures len(v) == len(old(v)) - 1;
        ensures result == old(v[len(v)-1]);
        ensures v == old(v[0..(len(v)-1)]);
    }

    // Destroy the vector, abort if not empty.
    fun verify_model_destroy_empty<Element>(v: vector<Element>) {
        Vector::destroy_empty(v); // inlining the built-in Boogie procedure
    }
    spec fun verify_model_destroy_empty {
        aborts_if len(v) > 0;
        // TODO: anything else?
    }

    // Swaps the elements at the i'th and j'th indices in the vector.
    fun verify_model_swap<Element>(v: &mut vector<Element>, i: u64, j: u64) {
        Vector::swap(v, i, j); // inlining the built-in Boogie procedure
    }
    spec fun verify_model_swap {
        aborts_if i >= len(v);
        aborts_if j >= len(v);
        ensures v == old(update_vector(update_vector(v,i,v[j]),j,v[i]));
    }

    // Return an vector of size one containing `e`
    fun verify_singleton<Element>(e: Element): vector<Element> {
        let v = Vector::empty();
        Vector::push_back(&mut v, e);
        v
    }
    spec fun verify_singleton {
        aborts_if false;
        ensures len(result) == 1;
        ensures result[0] == e;
    }

    spec module {
        /// Ghost variable `old_v` used to store the old value of the array v.
        /// TODO: by allowing old(.) to appear in assume/assert expressions,
        /// we can eliminate the need for old_v.
        global old_v<Element>: vector<Element>;
        global old_other<Element>: vector<Element>;
    }

    // Reverses the order of the elements in the vector in place.
    fun verify_reverse<Element>(v: &mut vector<Element>) {
        let vlen = Vector::length(v);
        if (vlen == 0) return ();

        spec {
            /// Initialize the old vector `old_v` ghost variable
            assume old_v<Element> == v;
        };

        let front_index = 0;
        let back_index = vlen -1;
        while ({
            spec {
                assert front_index + back_index == vlen - 1;
                assert forall i in 0..front_index: v[i] == old_v<Element>[vlen-1-i];
                assert forall i in 0..front_index: v[vlen-1-i] == old_v<Element>[i];
                assert forall j in front_index..back_index+1: v[j] == old_v<Element>[j];
                assert len(v) == vlen;
            };
            (front_index < back_index)
        }) {
            Vector::swap(v, front_index, back_index);
            front_index = front_index + 1;
            back_index = back_index - 1;
        };
    }
    spec fun verify_reverse {
        aborts_if false;
        ensures forall i in 0..len(v): v[i] == old(v)[len(v)-1-i];
    }

    // Reverses the order of the elements in the vector in place.
    fun verify_model_reverse<Element>(v: &mut vector<Element>) {
        Vector::reverse(v); // inlining the built-in Boogie procedure
    }
    spec fun verify_model_reverse {
        aborts_if false;
        ensures forall i in 0..len(v): old(v[i]) == v[len(v)-1-i];
    }

    // Moves all of the elements of the `other` vector into the `v` vector.
    fun verify_append<Element>(v: &mut vector<Element>, other: vector<Element>) {
        spec {
            /// Initialize the old vector `old_v` and `old_other` ghost variables
            assume old_v<Element> == v;
            assume old_other<Element> == other;
        };
        Vector::reverse(&mut other);
        while ({
            spec {
                assert len(v) >= len(old_v<Element>);
                assert len(other) <= len(old_other<Element>);
                assert len(v) + len(other) == len(old_v<Element>) + len(old_other<Element>);
                assert forall k in 0..len(old_v<Element>): v[k] == old_v<Element>[k];
                assert forall k in 0..len(other): other[k] == old_other<Element>[len(old_other<Element>)-1-k];
                assert forall k in len(old_v<Element>)..len(v): v[k] == old_other<Element>[k-len(old_v<Element>)];
            };
            !Vector::is_empty(&other)
        }) {
            Vector::push_back(v, Vector::pop_back(&mut other))
        };
        Vector::destroy_empty(other);
    }
    spec fun verify_append {
        ensures len(v) == old(len(v) + len(other));
        ensures v[0..len(old(v))] == old(v);
        ensures v[len(old(v))..len(v)] == old(other);
    }

    // Moves all of the elements of the `other` vector into the `lhs` vector.
    fun verify_model_append<Element>(lhs: &mut vector<Element>, other: vector<Element>) {
        Vector::append(lhs, other) // inlining the built-in Boogie procedure
    }
    spec fun verify_model_append {
        ensures len(lhs) == old(len(lhs) + len(other));
        ensures lhs[0..len(old(lhs))] == old(lhs);
        ensures lhs[len(old(lhs))..len(lhs)] == old(other);
    }

    // Return true if the vector has no elements
    fun verify_is_empty<Element>(v: &vector<Element>): bool {
        Vector::length(v) == 0
    }
    spec fun verify_is_empty {
        ensures result == (len(v) == 0);
    }

    // Return true if the vector has no elements
    fun verify_model_is_empty<Element>(v: &vector<Element>): bool {
        Vector::is_empty(v) // inlining the built-in Boogie procedure
    }
    spec fun verify_model_is_empty {
        ensures result == (len(v) == 0);
    }

    // Return (true, i) if `e` is in the vector `v` at index `i`.
    // Otherwise returns (false, 0).
    fun verify_index_of<Element>(v: &vector<Element>, e: &Element): (bool, u64) {
        let i = 0;
        let len = Vector::length(v);
        while ({
            spec {
                assert !(exists j in 0..i: v[j]==e);
            };
            i < len
        }) {
            if (Vector::borrow(v, i) == e) return (true, i);
            i = i + 1;
        };
        (false, 0)
    }
    spec fun verify_index_of {
        aborts_if false;
        ensures result_1 == (exists x in v: x==e); // whether v contains e or not
        ensures result_1 ==> v[result_2] == e; // if true, return the index where v contains e
        ensures result_1 ==> (forall i in 0..result_2: v[i]!=e); // ensure the smallest index
        ensures !result_1 ==> result_2 == 0; // return 0 if v does not contain e
    }

    fun verify_model_index_of<Element>(v: &vector<Element>, e: &Element): (bool, u64) {
        Vector::index_of(v, e) // inlining the built-in Boogie procedure
    }
    spec fun verify_model_index_of {
        aborts_if false;
        ensures result_1 == (exists x in v: x==e); // whether v contains e or not
        ensures result_1 ==> v[result_2] == e; // if true, return the index where v contains e
        ensures result_1 ==> (forall i in 0..result_2: v[i]!=e); // ensure the smallest index
        ensures !result_1 ==> result_2 == 0; // return 0 if v does not contain e
    }

    // Return true if `e` is in the vector `v`
    fun verify_contains<Element>(v: &vector<Element>, e: &Element): bool {
        let i = 0;
        let len = Vector::length(v);
        while ({
            spec {
               assert !(exists j in 0..i: v[j]==e);
            };
            i < len
        }) {
            if (Vector::borrow(v, i) == e) return true;
            i = i + 1;
        };
        spec {
           assert !(exists x in v: x==e);
        };
        false
    }
    spec fun verify_contains {
        aborts_if false;
        ensures result == (exists x in v: x==e);
    }

    // Return true if `e` is in the vector `v`
    fun verify_model_contains<Element>(v: &vector<Element>, e: &Element): bool {
        Vector::contains(v, e) // inlining the built-in Boogie procedure.
    }
    spec fun verify_model_contains {
        aborts_if false;
        ensures result == (exists x in v: x==e);
    }

    // Remove the `i`th element E of the vector, shifting all subsequent elements
    // It is O(n) and preserves ordering
    fun verify_remove<Element>(v: &mut vector<Element>, j: u64): Element {
        let vlen = Vector::length(v);
        let i = j;
        // i out of bounds; abort
        if (i >= vlen) abort 10;

        spec {
            /// Initialize the old vector `old_v` ghost variable
            assume old_v<Element> == v;
        };

        vlen = vlen - 1;
        while ({
            spec {
                assert j <= i && i <= vlen;
                assert vlen + 1 == len(v);
                assert v[0..j] == old_v<Element>[0..j];
                assert forall k in j..i: v[k] == old_v<Element>[k+1];
                assert forall k in i+1..len(v): v[k] == old_v<Element>[k];
                assert v[i] == old_v<Element>[j];
            };
            i < vlen
            }) {
            Vector::swap(v, i, { i = i + 1; i });
        };
        Vector::pop_back(v)
    }
    spec fun verify_remove {
        aborts_if j >= len(v);
        ensures len(v) == len(old(v)) - 1;
        ensures v[0..j] == old(v[0..j]);
        ensures v[j..len(v)] == old(v[j+1..len(v)]);
        ensures old(v[j]) == result;
    }

    // Remove the `i`th element E of the vector, shifting all subsequent elements
    // It is O(n) and preserves ordering
    fun verify_model_remove<Element>(v: &mut vector<Element>, i: u64): Element {
        Vector::remove(v, i) // inlining the built-in Boogie procedure.
    }
    spec fun verify_model_remove {
        aborts_if i >= len(v);
        ensures len(v) == len(old(v)) - 1;
        ensures v[0..i] == old(v[0..i]);
        ensures v[i..len(v)] == old(v[i+1..len(v)]);
        ensures old(v[i]) == result;
    }

    // Remove the `i`th element E of the vector by swapping it with the last element,
    // and then popping it off
    // It is O(1), but does not preserve ordering
    fun verify_swap_remove<Element>(v: &mut vector<Element>, i: u64): Element {
        let last_idx = Vector::length(v) - 1;
        Vector::swap(v, i, last_idx);
        Vector::pop_back(v)
    }
    spec fun verify_swap_remove {
        aborts_if i >= len(v);
        ensures len(v) == len(old(v)) - 1;
        ensures v == old(update_vector(v,i,v[len(v)-1])[0..len(v)-1]);
        ensures old(v[i]) == result;
    }

    // Remove the `i`th element E of the vector by swapping it with the last element,
    // and then popping it off
    // It is O(1), but does not preserve ordering
    fun verify_model_swap_remove<Element>(v: &mut vector<Element>, i: u64): Element {
        Vector::swap_remove(v, i) // inlining the built-in Boogie procedure.
    }
    spec fun verify_model_swap_remove {
        aborts_if i >= len(v);
        ensures len(v) == len(old(v)) - 1;
        ensures v == old(update_vector(v,i,v[len(v)-1])[0..len(v)-1]);
        ensures old(v[i]) == result;
    }
}

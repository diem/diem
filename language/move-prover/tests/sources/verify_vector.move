// dep: tests/sources/stdlib/modules/vector.move

// This file is created to verify the vector module in the standard library.
// This file is basically a clone of `stdlib/modules/vector.move` with renaming the module and function names.
// In this file, the functions with prefix of `verify_model` are verifying the corresponding built-in Boogie
// procedures that they inline (e.g., `verify_model_remove`).
// This file also attempts to verify the actual Move implementations of non-native functions (e.g., `verify_remove`),
// but it fails if loops are involved.

module VerifyVector {
    use 0x0::Vector;

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
        //ensures v[0..len(v)] == v;
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
        ensures v == old(update(update(v,i,v[j]),j,v[i]));
    }

    // Reverses the order of the elements in the vector in place.
    fun verify_reverse<Element>(v: &mut vector<Element>) {
        let len = Vector::length(v);
        if (len == 0) return ();

        let front_index = 0;
        let back_index = len -1;
        while (front_index < back_index) {
            Vector::swap(v, front_index, back_index);
            front_index = front_index + 1;
            back_index = back_index - 1;
        }
    }
    spec fun verify_reverse {
        // TODO: may need to extend the spec language to be able to specify this
    }

    // Reverses the order of the elements in the vector in place.
    fun verify_model_reverse<Element>(v: &mut vector<Element>) {
        Vector::reverse(v); // inlining the built-in Boogie procedure
    }
    spec fun verify_model_reverse {
        // TODO: may need to extend the spec language to be able to specify this
    }

    // Moves all of the elements of the `other` vector into the `lhs` vector.
    fun verify_append<Element>(lhs: &mut vector<Element>, other: vector<Element>) {
        Vector::reverse(&mut other);
        while (!Vector::is_empty(&other)) Vector::push_back(lhs, Vector::pop_back(&mut other));
        Vector::destroy_empty(other);
    }
    spec fun verify_append { // TODO: cannot verify loop
        //ensures len(lhs) == old(len(lhs) + len(other)); //! A postcondition might not hold on this return path.
        //ensures lhs[0..len(old(lhs))] == old(lhs); //! A postcondition might not hold on this return path.
        //ensures lhs[len(old(lhs))..len(lhs)] == old(other); //! A postcondition might not hold on this return path.
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

    // Return true if `e` is in the vector `v`
    fun verify_contains<Element>(v: &vector<Element>, e: &Element): bool {
        let i = 0;
        let len = Vector::length(v);
        while (i < len) {
            if (Vector::borrow(v, i) == e) return true;
            i = i + 1;
        };
        false
    }
    spec fun verify_contains { // TODO: cannot verify loop
        //ensures any(v,|x| x==e); //! A postcondition might not hold on this return path.
    }

    // Return true if `e` is in the vector `v`
    fun verify_model_contains<Element>(v: &vector<Element>, e: &Element): bool {
        Vector::contains(v, e) // inlining the built-in Boogie procedure.
    }
    spec fun verify_model_contains {
        aborts_if false;
        ensures result == any(v,|x| x==e);
    }

    // Remove the `i`th element E of the vector, shifting all subsequent elements
    // It is O(n) and preserves ordering
    fun verify_remove<Element>(v: &mut vector<Element>, i: u64): Element {
        let len = Vector::length(v);
        // i out of bounds; abort
        if (i >= len) abort 10;

        len = len - 1;
        while (i < len) Vector::swap(v, i, { i = i + 1; i });
        Vector::pop_back(v)
    }
    spec fun verify_remove { // TODO: cannot verify loop
        //aborts_if i >= len(old(v)); //! A postcondition might not hold on this return path.
        //ensures len(v) == len(old(v)) - 1; //! A postcondition might not hold on this return path.
        //ensures v[0..i] == old(v[0..i]); //! A postcondition might not hold on this return path.
        //ensures v[i..len(v)] == old(v[i+1..len(v)]); //! A postcondition might not hold on this return path.
    }

    // Remove the `i`th element E of the vector, shifting all subsequent elements
    // It is O(n) and preserves ordering
    fun verify_model_remove<Element>(v: &mut vector<Element>, i: u64): Element {
        Vector::remove(v, i) // inlining the built-in Boogie procedure.
    }
    spec fun verify_model_remove {
        aborts_if i >= len(old(v));
        ensures len(v) == len(old(v)) - 1;
        ensures v[0..i] == old(v[0..i]);
        ensures v[i..len(v)] == old(v[i+1..len(v)]);
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
        aborts_if i >= len(old(v));
        ensures len(v) == len(old(v)) - 1;
        ensures v == old(update(v,i,v[len(v)-1])[0..len(v)-1]);
    }

    // Remove the `i`th element E of the vector by swapping it with the last element,
    // and then popping it off
    // It is O(1), but does not preserve ordering
    fun verify_model_swap_remove<Element>(v: &mut vector<Element>, i: u64): Element {
        Vector::swap_remove(v, i) // inlining the built-in Boogie procedure.
    }
    spec fun verify_model_swap_remove {
        aborts_if i >= len(old(v));
        ensures len(v) == len(old(v)) - 1;
        ensures v == old(update(v,i,v[len(v)-1])[0..len(v)-1]);
    }
}

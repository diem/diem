// This file is created to verify the vector module in the standard library.
// This file is basically a clone of `stdlib/modules/vector.move` with renaming the module and function names.
// In this file, the functions with prefix of `verify_model` are verifying the corresponding built-in Boogie
// procedures that they inline (e.g., `verify_model_remove`).
// This file also attempts to verify the actual Move implementations of non-native functions (e.g., `verify_remove`),
// but it fails if loops are involved.

module VerifyVector {
    use 0x0::Vector;

    spec module {
        pragma verify = true;
    }

    // Return true if the vector has no elements
    fun verify_is_empty<Element>(v: &vector<Element>): bool {
        Vector::length(v) == 0
    }
    spec fun verify_is_empty {
        ensures result == (len(v) == 0);
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
        ensures old(v[i]) == result;
    }
}

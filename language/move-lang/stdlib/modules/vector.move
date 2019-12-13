address 0x0:

// A variable-sized container that can hold both unrestricted types and resources.
module Vector {

    // Vector containing data of type Item.
    native struct T<Element>;

    native public empty<Element>(): T<Element>;

    // Return the length of the vector.
    native public length<Element>(v: &T<Element>): u64;

    // Acquire an immutable reference to the ith element of the vector.
    native public borrow<Element>(v: &T<Element>, i: u64): &Element;

    // Add an element to the end of the vector.
    native public push_back<Element>(v: &mut T<Element>, e: Element);

    // Get mutable reference to the ith element in the vector, abort if out of bound.
    native public borrow_mut<Element>(v: &mut T<Element>, idx: u64): &mut Element;

    // Pop an element from the end of vector, abort if the vector is empty.
    native public pop_back<Element>(v: &mut T<Element>): Element;

    // Destroy the vector, abort if not empty.
    native public destroy_empty<Element>(v: T<Element>);

    // Swaps the elements at the i'th and j'th indices in the vector.
    native public swap<Element>(v: &mut T<Element>, i: u64, j: u64);

    // Reverses the order of the elements in the vector in place.
    public reverse<Element>(v: &mut T<Element>) {

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
    public append<Element>(lhs: &mut T<Element>, other: T<Element>) {
        reverse(&mut other);
        while (!is_empty(&other)) push_back(lhs, pop_back(&mut other));
        destroy_empty(other);
    }

    // Return true if the vector has no elements
    public is_empty<Element>(v: &T<Element>): bool {
        length(v) == 0
    }

}

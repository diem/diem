address 0x1 {

/**
This module defines the Option type and its methods to represent and handle an optional value.
*/
module Option {
    use 0x1::Vector;

    // Abstraction of a value that may or may not be present. Implemented with a vector of size
    // zero or one because Move bytecode does not have ADTs.
    struct Option<Element> { vec: vector<Element> }

    // Return an empty `Option`
    public fun none<Element>(): Option<Element> {
        Option { vec: Vector::empty() }
    }

    // Return an `Option` containing `e`
    public fun some<Element>(e: Element): Option<Element> {
        Option { vec: Vector::singleton(e) }
    }

    // Return true if `t` does not hold a value
    public fun is_none<Element>(t: &Option<Element>): bool {
        Vector::is_empty(&t.vec)
    }

    // Return true if `t` holds a value
    public fun is_some<Element>(t: &Option<Element>): bool {
        !Vector::is_empty(&t.vec)
    }

    // Return true if the value in `t` is equal to `e_ref`
    // Always returns `false` if `t` does not hold a value
    public fun contains<Element>(t: &Option<Element>, e_ref: &Element): bool {
        Vector::contains(&t.vec, e_ref)
    }

    // Return an immutable reference to the value inside `t`
    // Aborts if `t` does not hold a value
    public fun borrow<Element>(t: &Option<Element>): &Element {
        Vector::borrow(&t.vec, 0)
    }

    // Return a reference to the value inside `t` if it holds one
    // Return `default_ref` if `t` does not hold a value
    public fun borrow_with_default<Element>(t: &Option<Element>, default_ref: &Element): &Element {
        let vec_ref = &t.vec;
        if (Vector::is_empty(vec_ref)) default_ref
        else Vector::borrow(vec_ref, 0)
    }

    // Return the value inside `t` if it holds one
    // Return `default` if `t` does not hold a value
    public fun get_with_default<Element: copyable>(t: &Option<Element>, default: Element): Element {
        let vec_ref = &t.vec;
        if (Vector::is_empty(vec_ref)) default
        else *Vector::borrow(vec_ref, 0)
    }

    // Convert the none option `t` to a some option by adding `e`.
    // Aborts if `t` already holds a value
    public fun fill<Element>(t: &mut Option<Element>, e: Element) {
        let vec_ref = &mut t.vec;
        if (Vector::is_empty(vec_ref)) Vector::push_back(vec_ref, e)
        else abort(99)
    }

    // Convert a `some` option to a `none` by removing and returning the value stored inside `t`
    // Aborts if `t` does not hold a value
    public fun extract<Element>(t: &mut Option<Element>): Element {
        Vector::pop_back(&mut t.vec)
    }

    // Return a mutable reference to the value inside `t`
    // Aborts if `t` does not hold a value
    public fun borrow_mut<Element>(t: &mut Option<Element>): &mut Element {
        Vector::borrow_mut(&mut t.vec, 0)
    }

    // Swap the old value inside `t` with `e` and return the old value
    // Aborts if `t` does not hold a value
    public fun swap<Element>(t: &mut Option<Element>, e: Element): Element {
        let vec_ref = &mut t.vec;
        let old_value = Vector::pop_back(vec_ref);
        Vector::push_back(vec_ref, e);
        old_value
    }

    // Destroys `t.` If `t` holds a value, return it. Returns `default` otherwise
    public fun destroy_with_default<Element: copyable>(t: Option<Element>, default: Element): Element {
        let Option { vec } = t;
        if (Vector::is_empty(&mut vec)) default
        else Vector::pop_back(&mut vec)
    }

    // Unpack `t` and return its contents
    // Aborts if `t` does not hold a value
    public fun destroy_some<Element>(t: Option<Element>): Element {
        let Option { vec } = t;
        let elem = Vector::pop_back(&mut vec);
        Vector::destroy_empty(vec);
        elem
    }

    // Unpack `t`
    // Aborts if `t` holds a value
    public fun destroy_none<Element>(t: Option<Element>) {
        let Option { vec } = t;
        Vector::destroy_empty(vec)
    }


    // ****************** SPECIFICATIONS *******************

    spec struct Option {
        /// The size of vector is always less than equal to 1
        /// because it's 0 for "none" or 1 for "some".
        invariant len(vec) <= 1;
    }

    spec module {
        /// Return true iff t contains none.
        define is_none_spec<Element>(t: Option<Element>): bool {
            len(t.vec) == 0
        }
        /// Return true iff t contains some.
        define is_some_spec<Element>(t: Option<Element>): bool {
            !is_none_spec(t)
        }
        /// Return the value inside of t.
        define value_inside<Element>(t: Option<Element>): Element {
            t.vec[0]
        }
    }

    spec fun none {
        aborts_if false;
        ensures is_none_spec(result);
    }

    spec fun some {
        aborts_if false;
        ensures is_some_spec(result);
        ensures value_inside(result) == e;
    }

    spec fun is_none {
        aborts_if false;
        ensures result == is_none_spec(t);
    }

    spec fun is_some {
        aborts_if false;
        ensures result == is_some_spec(t);
    }

    spec fun contains {
        aborts_if false;
        ensures result == (is_some_spec(t) && value_inside(t) == e_ref);
    }

    spec fun borrow {
        aborts_if is_none_spec(t);
        ensures result == value_inside(t);
    }

    spec fun borrow_with_default {
        aborts_if false;
        ensures is_none_spec(t) ==> result == default_ref;
        ensures is_some_spec(t) ==> result == value_inside(t);
    }

    spec fun get_with_default {
        aborts_if false;
        ensures is_none_spec(t) ==> result == default;
        ensures is_some_spec(t) ==> result == value_inside(t);
    }

    spec fun fill {
        aborts_if is_some_spec(t);
        ensures is_some_spec(t);
        ensures value_inside(t) == e;
    }

    spec fun extract {
        aborts_if is_none_spec(t);
        ensures result == value_inside(old(t));
        ensures is_none_spec(t);
    }

    spec fun borrow_mut {
        aborts_if is_none_spec(t);
        ensures result == value_inside(t);
    }

    spec fun swap {
        aborts_if is_none_spec(t);
        ensures result == value_inside(old(t));
        ensures is_some_spec(t);
        ensures value_inside(t) == e;
    }

    spec fun destroy_with_default {
        aborts_if false;
        ensures is_none_spec(old(t)) ==> result == default;
        ensures is_some_spec(old(t)) ==> result == value_inside(old(t));
    }

    spec fun destroy_some {
        aborts_if is_none_spec(t);
        ensures result == value_inside(old(t));
    }

    spec fun destroy_none {
        aborts_if is_some_spec(t);
    }
}

}

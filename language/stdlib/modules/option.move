address 0x0 {
module Option {
    use 0x0::Vector;

    // Abstraction of a value that may or may not be present. Implemented with a vector of size
    // zero or one because Move bytecode does not have ADTs.
    struct T<Element> { vec: vector<Element> }

    // Return an empty `Option::T`
    public fun none<Element>(): T<Element> {
        T { vec: Vector::empty() }
    }

    // Return an `Option::T` containing `e`
    public fun some<Element>(e: Element): T<Element> {
        T { vec: Vector::singleton(e) }
    }

    // Return true if `t` does not hold a value
    public fun is_none<Element>(t: &T<Element>): bool {
        Vector::is_empty(&t.vec)
    }

    // Return true if `t` holds a value
    public fun is_some<Element>(t: &T<Element>): bool {
        !Vector::is_empty(&t.vec)
    }

    // Return true if the value in `t` is equal to `e_ref`
    // Always returns `false` if `t` does not hold a value
    public fun contains<Element>(t: &T<Element>, e_ref: &Element): bool {
        Vector::contains(&t.vec, e_ref)
    }

    // Return an immutable reference to the value inside `t`
    // Aborts if `t` does not hold a value
    public fun borrow<Element>(t: &T<Element>): &Element {
        Vector::borrow(&t.vec, 0)
    }

    // Return a reference to the value inside `t` if it holds one
    // Return `default_ref` if `t` does not hold a value
    public fun borrow_with_default<Element>(t: &T<Element>, default_ref: &Element): &Element {
        let vec_ref = &t.vec;
        if (Vector::is_empty(vec_ref)) default_ref
        else Vector::borrow(vec_ref, 0)
    }

    // Return the value inside `t` if it holds one
    // Return `default` if `t` does not hold a value
    public fun get_with_default<Element: copyable>(t: &T<Element>, default: Element): Element {
        let vec_ref = &t.vec;
        if (Vector::is_empty(vec_ref)) default
        else *Vector::borrow(vec_ref, 0)
    }

    // Convert the none option `t` to a some option by adding `e`.
    // Aborts if `t` already holds a value
    public fun fill<Element>(t: &mut T<Element>, e: Element) {
        let vec_ref = &mut t.vec;
        if (Vector::is_empty(vec_ref)) Vector::push_back(vec_ref, e)
        else abort(99)
    }

    // Convert a `some` option to a `none` by removing and returning the value stored inside `t`
    // Aborts if `t` does not hold a value
    public fun extract<Element>(t: &mut T<Element>): Element {
        Vector::pop_back(&mut t.vec)
    }

    // Return a mutable reference to the value inside `t`
    // Aborts if `t` does not hold a value
    public fun borrow_mut<Element>(t: &mut T<Element>): &mut Element {
        Vector::borrow_mut(&mut t.vec, 0)
    }

    // Swap the old value inside `t` with `e` and return the old value
    // Aborts if `t` does not hold a value
    public fun swap<Element>(t: &mut T<Element>, e: Element): Element {
        let vec_ref = &mut t.vec;
        let old_value = Vector::pop_back(vec_ref);
        Vector::push_back(vec_ref, e);
        old_value
    }

    // Destroys `t.` If `t` holds a value, return it. Returns `default` otherwise
    public fun destroy_with_default<Element: copyable>(t: T<Element>, default: Element): Element {
        let T { vec } = t;
        if (Vector::is_empty(&mut vec)) default
        else Vector::pop_back(&mut vec)
    }

    // Unpack `t` and return its contents
    // Aborts if `t` does not hold a value
    public fun destroy_some<Element>(t: T<Element>): Element {
        let T { vec } = t;
        let elem = Vector::pop_back(&mut vec);
        Vector::destroy_empty(vec);
        elem
    }

    // Unpack `t`
    // Aborts if `t` holds a value
    public fun destroy_none<Element>(t: T<Element>) {
        let T { vec } = t;
        Vector::destroy_empty(vec)
    }

}

}

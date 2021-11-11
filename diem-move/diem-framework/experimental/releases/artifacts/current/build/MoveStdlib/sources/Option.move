/// This module defines the Option type and its methods to represent and handle an optional value.
module Std::Option {
    use Std::Errors;
    use Std::Vector;

    /// Abstraction of a value that may or may not be present. Implemented with a vector of size
    /// zero or one because Move bytecode does not have ADTs.
    struct Option<Element> has copy, drop, store {
        vec: vector<Element>
    }
    spec Option {
        /// The size of vector is always less than equal to 1
        /// because it's 0 for "none" or 1 for "some".
        invariant len(vec) <= 1;
    }

    /// The `Option` is in an invalid state for the operation attempted.
    /// The `Option` is `Some` while it should be `None`.
    const EOPTION_IS_SET: u64 = 0;
    /// The `Option` is in an invalid state for the operation attempted.
    /// The `Option` is `None` while it should be `Some`.
    const EOPTION_NOT_SET: u64 = 1;

    /// Return an empty `Option`
    public fun none<Element>(): Option<Element> {
        Option { vec: Vector::empty() }
    }
    spec none {
        pragma opaque;
        aborts_if false;
        ensures result == spec_none<Element>();
    }
    spec fun spec_none<Element>(): Option<Element> {
        Option{ vec: vec() }
    }

    /// Return an `Option` containing `e`
    public fun some<Element>(e: Element): Option<Element> {
        Option { vec: Vector::singleton(e) }
    }
    spec some {
        pragma opaque;
        aborts_if false;
        ensures result == spec_some(e);
    }
    spec fun spec_some<Element>(e: Element): Option<Element> {
        Option{ vec: vec(e) }
    }

    /// Return true if `t` does not hold a value
    public fun is_none<Element>(t: &Option<Element>): bool {
        Vector::is_empty(&t.vec)
    }
    spec is_none {
        pragma opaque;
        aborts_if false;
        ensures result == is_none(t);
    }

    /// Return true if `t` holds a value
    public fun is_some<Element>(t: &Option<Element>): bool {
        !Vector::is_empty(&t.vec)
    }
    spec is_some {
        pragma opaque;
        aborts_if false;
        ensures result == is_some(t);
    }

    /// Return true if the value in `t` is equal to `e_ref`
    /// Always returns `false` if `t` does not hold a value
    public fun contains<Element>(t: &Option<Element>, e_ref: &Element): bool {
        Vector::contains(&t.vec, e_ref)
    }
    spec contains {
        pragma opaque;
        aborts_if false;
        ensures result == spec_contains(t, e_ref);
    }
    spec fun spec_contains<Element>(t: Option<Element>, e: Element): bool {
        is_some(t) && borrow(t) == e
    }

    /// Return an immutable reference to the value inside `t`
    /// Aborts if `t` does not hold a value
    public fun borrow<Element>(t: &Option<Element>): &Element {
        assert!(is_some(t), Errors::invalid_argument(EOPTION_NOT_SET));
        Vector::borrow(&t.vec, 0)
    }
    spec borrow {
        pragma opaque;
        include AbortsIfNone<Element>;
        ensures result == borrow(t);
    }

    /// Return a reference to the value inside `t` if it holds one
    /// Return `default_ref` if `t` does not hold a value
    public fun borrow_with_default<Element>(t: &Option<Element>, default_ref: &Element): &Element {
        let vec_ref = &t.vec;
        if (Vector::is_empty(vec_ref)) default_ref
        else Vector::borrow(vec_ref, 0)
    }
    spec borrow_with_default {
        pragma opaque;
        aborts_if false;
        ensures result == (if (is_some(t)) borrow(t) else default_ref);
    }

    /// Return the value inside `t` if it holds one
    /// Return `default` if `t` does not hold a value
    public fun get_with_default<Element: copy + drop>(
        t: &Option<Element>,
        default: Element,
    ): Element {
        let vec_ref = &t.vec;
        if (Vector::is_empty(vec_ref)) default
        else *Vector::borrow(vec_ref, 0)
    }
    spec get_with_default {
        pragma opaque;
        aborts_if false;
        ensures result == (if (is_some(t)) borrow(t) else default);
    }

    /// Convert the none option `t` to a some option by adding `e`.
    /// Aborts if `t` already holds a value
    public fun fill<Element>(t: &mut Option<Element>, e: Element) {
        let vec_ref = &mut t.vec;
        if (Vector::is_empty(vec_ref)) Vector::push_back(vec_ref, e)
        else abort Errors::invalid_argument(EOPTION_IS_SET)
    }
    spec fill {
        pragma opaque;
        aborts_if is_some(t) with Errors::INVALID_ARGUMENT;
        ensures is_some(t);
        ensures borrow(t) == e;
    }

    /// Convert a `some` option to a `none` by removing and returning the value stored inside `t`
    /// Aborts if `t` does not hold a value
    public fun extract<Element>(t: &mut Option<Element>): Element {
        assert!(is_some(t), Errors::invalid_argument(EOPTION_NOT_SET));
        Vector::pop_back(&mut t.vec)
    }
    spec extract {
        pragma opaque;
        include AbortsIfNone<Element>;
        ensures result == borrow(old(t));
        ensures is_none(t);
    }

    /// Return a mutable reference to the value inside `t`
    /// Aborts if `t` does not hold a value
    public fun borrow_mut<Element>(t: &mut Option<Element>): &mut Element {
        assert!(is_some(t), Errors::invalid_argument(EOPTION_NOT_SET));
        Vector::borrow_mut(&mut t.vec, 0)
    }
    spec borrow_mut {
        pragma opaque;
        include AbortsIfNone<Element>;
        ensures result == borrow(t);
    }

    /// Swap the old value inside `t` with `e` and return the old value
    /// Aborts if `t` does not hold a value
    public fun swap<Element>(t: &mut Option<Element>, e: Element): Element {
        assert!(is_some(t), Errors::invalid_argument(EOPTION_NOT_SET));
        let vec_ref = &mut t.vec;
        let old_value = Vector::pop_back(vec_ref);
        Vector::push_back(vec_ref, e);
        old_value
    }
    spec swap {
        pragma opaque;
        include AbortsIfNone<Element>;
        ensures result == borrow(old(t));
        ensures is_some(t);
        ensures borrow(t) == e;
    }

    /// Destroys `t.` If `t` holds a value, return it. Returns `default` otherwise
    public fun destroy_with_default<Element: drop>(t: Option<Element>, default: Element): Element {
        let Option { vec } = t;
        if (Vector::is_empty(&mut vec)) default
        else Vector::pop_back(&mut vec)
    }
    spec destroy_with_default {
        pragma opaque;
        aborts_if false;
        ensures result == (if (is_some(t)) borrow(t) else default);
    }

    /// Unpack `t` and return its contents
    /// Aborts if `t` does not hold a value
    public fun destroy_some<Element>(t: Option<Element>): Element {
        assert!(is_some(&t), Errors::invalid_argument(EOPTION_NOT_SET));
        let Option { vec } = t;
        let elem = Vector::pop_back(&mut vec);
        Vector::destroy_empty(vec);
        elem
    }
    spec destroy_some {
        pragma opaque;
        include AbortsIfNone<Element>;
        ensures result == borrow(t);
    }

    /// Unpack `t`
    /// Aborts if `t` holds a value
    public fun destroy_none<Element>(t: Option<Element>) {
        assert!(is_none(&t), Errors::invalid_argument(EOPTION_IS_SET));
        let Option { vec } = t;
        Vector::destroy_empty(vec)
    }
    spec destroy_none {
        pragma opaque;
        aborts_if is_some(t) with Errors::INVALID_ARGUMENT;
    }

    spec module {} // switch documentation context back to module level

    spec module {
        pragma aborts_if_is_strict;
    }

    /// # Helper Schema

    spec schema AbortsIfNone<Element> {
        t: Option<Element>;
        aborts_if is_none(t) with Errors::INVALID_ARGUMENT;
    }
}

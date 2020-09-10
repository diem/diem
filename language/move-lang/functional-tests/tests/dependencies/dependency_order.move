// names used to try to force an ordering of depedencies
module C {
    struct T {}
    public fun foo(): T {
        T{}
    }
}

//! new-transaction
// names used to try to force an ordering of depedencies
module B {
    public fun foo(): {{default}}::C::T {
        {{default}}::C::foo()
    }
}

//! new-transaction
module A {
    struct T {
        t_b: {{default}}::C::T,
        t_c: {{default}}::C::T,
    }
    public fun foo(): T {
        T {
            t_c: {{default}}::C::foo(),
            t_b: {{default}}::B::foo()
        }
    }
}

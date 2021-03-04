address 0x42 {
module C {
    native struct T;
}

module B {
    use 0x42::C;
    public fun foo(): C::T {
        C::T {}
    }
    public fun bar(c: C::T) {
        let C::T {} = c;
    }
    public fun baz(c: C::T) {
        let f = c.f;
    }
}
}

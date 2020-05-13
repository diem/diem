module TestNestedInvariants {
    struct Nested {
        x: u64
    }

    struct Outer {
        y: u64,
        n: Nested
    }

    fun new(): Outer {
        Outer{y: 3, n: Nested{x: 2}}
    }

    fun new_outer_data_invariant_invalid(): Outer {
        Outer{y: 2, n: Nested{x: 2}}
    }

    fun new_inner_data_invariant_invalid(): Outer {
        Outer{y: 2, n: Nested{x: 0}}
    }

    fun mutate() {
        let o = Outer{y: 3, n: Nested{x: 2}};
        let r = &mut o;
        r.y = 2;
        r.n.x = 1;
    }

    fun mutate_outer_data_invariant_invalid() {
        let o = Outer{y: 3, n: Nested{x: 2}};
        let r = &mut o;
        r.y = 2;
    }

    fun mutate_inner_data_invariant_invalid() {
        let o = Outer{y: 3, n: Nested{x: 2}};
        let r = &mut o;
        r.n.x = 0;
    }
}

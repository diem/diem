module 0x1::NestedFields {
    struct A { b: B }
    struct B { c: C }
    struct C { f: address }

    fun nested_fields_direct(a: &A): address {
        a.b.c.f
    }

    fun nested_fields_helper1(c: &C): address {
        c.f
    }

    fun nested_fields_helper2(b: &B): address {
        b.c.f
    }

    fun nested_fields_interproc(a1: &A, a2: &A, flag: bool): address {
        if (flag) {
            nested_fields_helper1(&a1.b.c)
        } else {
            nested_fields_helper2(&a2.b)
        }
    }
}

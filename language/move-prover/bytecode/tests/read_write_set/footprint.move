address 0x2 {
module Footprint {
    struct S { f: address }

    // expected: empty summary
    public fun reassign_constant(a: address): address {
        a = @0x2;
        a
    }

    // expected: returns Footprint(a2)
    public fun reassign_other_param(a1: address, a2: address): address {
        a1 = a2;
        a1
    }

    // expected: returns Footprint({a, 0x2})
    public fun reassign_cond(a: address, b: bool): address {
        if (b) {
            a = @0x2;
        };
        _ = 2 + 2;
        a
    }

    // expected: s.f |-> 0x2
    public fun reassign_field(s: &mut S) {
        s.f = @0x2;
    }

    // expected: s.f |-> {0x2, Footprint(s.f)}
    public fun reassign_field_cond(s: &mut S, b: bool) {
        if (b) {
            s.f = @0x2
        }
    }
}
}

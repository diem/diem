module M {
    resource struct R {}

    t0(): u64 {
        ()
    }

    t1(): () {
        0
    }

    t2(): (u64, bool) {
        (0, false, R{})
    }

    t3(): (u64, bool, R, bool) {
        (0, false, R{})
    }

    t4(): (bool, u64, R) {
        (0, false, R{})
    }
}

module M {
    resource struct R {}

    t0(): () {
        ()
    }

    t1(): u64 {
        0
    }

    t2(): (u64, bool, R) {
        (0, false, R{})
    }

}

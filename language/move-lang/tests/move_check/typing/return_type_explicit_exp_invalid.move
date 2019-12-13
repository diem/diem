module M {
    resource struct R {}

    t0(): u64 {
        return ()
    }

    t1(): () {
        if (true) return 1 else return 0
    }

    t2(): (u64, bool) {
        loop return (0, false, R{});
        abort 0
    }

    t3(): (u64, bool, R, bool) {
        while (true) return (0, false, R{});
        abort 0
    }

    t4(): (bool, u64, R) {
        while (false) return (0, false, R{});
        abort 0
    }
}

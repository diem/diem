module M {
    resource struct R {}

    t0(): () {
        if (true) return () else return ()
    }

    t1(): u64 {
        return 0
    }

    t2(): (u64, bool, R) {
        while (true) return (0, false, R{});
        abort 0
    }
}

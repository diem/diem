address 0x1:

module X {
    resource struct R {}
}

module M {
    use 0x1::X;

    fun t0(): X::R {
        loop { if (false) break }
    }

    fun t1(): u64 {
        loop { let x = 0; break }
    }

    fun t2() {
        foo(loop { break })
    }

    fun foo(x: u64) {}

    fun t3() {
        let x = loop { break };
        let (x, y) = loop { if (false) break };
    }
}

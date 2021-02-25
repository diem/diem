address 0x2 {

module X {
    struct R {}
}

module M {
    use 0x2::X;

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

}

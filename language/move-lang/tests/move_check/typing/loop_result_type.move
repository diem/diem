address 0x1:

module X {
    resource struct R {}
}

module M {
    use 0x1::X;

    fun t0(): X::R {
        loop {}
    }

    fun t1(): u64 {
        loop { let x = 0; x; }
    }

    fun t2() {
        foo(loop {})
    }

    fun foo(_x: u64) {}

    fun t3(): X::R {
        let x: X::R = loop { 0; };
        x
    }

    fun t4() {
        let () = loop { break };
        let () = loop { if (false) break };
    }

}

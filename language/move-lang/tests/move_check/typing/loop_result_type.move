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

    fun foo(x: u64) {}

    fun t3() {
        let x = loop { 0; };
    }

    fun t4() {
        let () = loop { break };
        let () = loop { if (false) break };
    }

}

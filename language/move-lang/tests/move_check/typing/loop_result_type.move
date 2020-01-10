address 0x1:

module X {
    resource struct R {}
}

module M {
    use 0x1::X;

    t0(): X::R {
        loop {}
    }

    t1(): u64 {
        loop { let x = 0; x; }
    }

    t2() {
        foo(loop {})
    }

    foo(x: u64) {}

    t3() {
        let x = loop { 0; };
    }

    t4() {
        let () = loop { break };
        let () = loop { if (false) break };
    }

}

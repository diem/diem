address 0x1:

module X {
    resource struct R {}
}

module M {
    use 0x1::X;

    t0(): X::R {
        loop { if (false) break }
    }

    t1(): u64 {
        loop { let x = 0; break }
    }

    t2() {
        foo(loop { break })
    }

    foo(x: u64) {}

    t3() {
        let x = loop { break };
        let (x, y) = loop { if (false) break };
    }
}

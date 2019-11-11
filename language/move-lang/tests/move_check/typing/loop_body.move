module M {
    t0() {
        loop ()
    }

    t1() {
        loop (())
    }

    t2() {
        loop {}
    }

    t3() {
        loop { let x = 0; x; }
    }

    t4() {
        loop { if (true) () }
    }

    t5() {
        loop break;
        loop { break };
        loop return ()
    }

    t6() {
        loop continue
    }

    t7() {
        loop { continue }
    }

    t8() {
        loop { loop { break } }
    }
}

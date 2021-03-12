module 0x8675309::M {
    fun t0() {
        loop ()
    }

    fun t1() {
        loop (())
    }

    fun t2() {
        loop {}
    }

    fun t3() {
        loop { let x = 0; x; }
    }

    fun t4() {
        loop { if (true) () }
    }

    fun t5() {
        loop break;
        loop { break };
        loop return ()
    }

    fun t6() {
        loop continue
    }

    fun t7() {
        loop { continue }
    }

    fun t8() {
        loop { loop { break } }
    }
}

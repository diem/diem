module 0x8675309::M {
    fun t0() {
        loop 0
    }

    fun t1() {
        loop false
    }

    fun t2() {
        loop { @0x0 }
    }

    fun t3() {
        loop { let x = 0; x }
    }

    fun t4() {
        loop { if (true) 1 else 0 }
    }
}

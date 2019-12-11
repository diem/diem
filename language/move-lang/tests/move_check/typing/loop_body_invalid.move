module M {
    t0() {
        loop 0
    }

    t1() {
        loop false
    }

    t2() {
        loop { 0x0 }
    }

    t3() {
        loop { let x = 0; x }
    }

    t4() {
        loop { if (true) 1 else 0 }
    }
}

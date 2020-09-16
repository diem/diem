module TestSpecBlock {

    fun simple1(x: u64, y: u64) {
        if (!(x > y)) abort 1;
        spec {
            assert x > y;
        }
    }

    fun simple2(x: u64) {
        let y: u64;
        y = x + 1;
        spec {
            assert x == y - 1;
        }
    }

    fun simple3(x: u64, y: u64) {
        spec {
            assume x > y;
            assert x >= y;
        }
    }

    fun simple4(x: u64, y: u64) {
        let z: u64;
        z = x + y;
        spec {
            assume x > y;
            assert z > 2*y;
        }
    }

    fun looping(x: u64): u64 {
        spec { assume x <= 10; };
        while (x < 10) {
          spec { assume x < 10; };
          x = x + 1;
        };
        spec { assert x == 10; };
        x
    }
}

module 0x2::A {
    #[test]
    public fun check_lambda() {
        spec {
            assert f2() == 2;
        };
    }
    spec module {
        fun f1(f: |u64| u64): u64 {
            f(1u64)
        }
        fun f2(): num {
            f1(|x| x + 1)
        }
    }
}

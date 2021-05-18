module 0x2::A {
    #[test]
    public fun check_const_pass() {
        spec {
            assert true;
        };
    }

    #[test]
    public fun check_const_fail() {
        spec {
            assert false;
        };
    }

    #[test]
    public fun check_local_pass(): bool {
        let a = true;
        spec {
            assert a;
        };
        a
    }

    #[test]
    public fun check_local_fail(): bool {
        let a = false;
        spec {
            assert a;
        };
        a
    }
}

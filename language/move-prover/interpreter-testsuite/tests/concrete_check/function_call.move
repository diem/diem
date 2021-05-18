module 0x2::A {
    public fun handle_val(x: u64): u64 {
        x + 1
    }

    public fun handle_imm_ref(x: &u64): u64 {
        *x + 1
    }

    public fun handle_mut_ref(x: &mut u64) {
        *x = 1;
    }

    public fun return_mut_ref(x: &mut u64): &mut u64 {
        x
    }

    #[test]
    public fun call_val(): u64 {
        let a = 0u64;
        handle_val(a)
    }

    #[test]
    public fun call_imm_ref(): u64 {
        let a = 0u64;
        let b = &a;
        handle_imm_ref(b)
    }

    #[test]
    public fun call_mut_ref(): u64 {
        let a = 0u64;
        let b = &mut a;
        handle_mut_ref(b);
        a
    }

    #[test]
    public fun call_return_mut_ref(): u64 {
        let a = 0u64;
        let b = &mut a;
        let c = return_mut_ref(b);
        handle_mut_ref(c);
        a
    }
}

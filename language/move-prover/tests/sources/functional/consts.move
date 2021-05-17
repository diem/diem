module 0x42::TestConst {

    struct T {
        x: u64,
        b: bool,
        a: address,
    }
    const INIT_VAL_U64: u64 = 40 + 2;
    const FORTY_TWO: u64 = 42;
    const INIT_VAL_BOOL: bool = true;
    const ONE: u64 = 1;
    const ADDR: address = @0x2;

    public fun init(): T {
        T { x: 43, b: !INIT_VAL_BOOL, a: ADDR }
    }

    spec init {
        ensures result.x == INIT_VAL_U64 + 1;
        ensures !result.b;
    }

    public fun init_incorrect(): T {
        T { x: 43, b: INIT_VAL_BOOL, a: @0x1 }
    }

    spec init_incorrect {
        ensures result.x == FORTY_TWO + ONE;
        ensures !result.b;
    }

}

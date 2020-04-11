module M {
    struct S { f: u64, g: bool }

    fun t0() {
        let x: u64;
    }

    fun t1() {
        let (x, y): (u64, u64);
    }

    fun t2() {
        let S{ f, g }: S;
    }

    fun unused_local_suppressed1() {
        let _x: u64;
    }

    fun unused_local_suppressed2() {
        let _: u64;
    }


    fun unused_param(x: u64) {
    }

    fun two_unused(x: u64, y: bool) {
    }

    fun unused_param1_used_param2(x: u64, y: bool): bool {
        y
    }

    fun unused_param2_used_param1(x: u64, y: bool): u64 {
        x
    }

    fun unused_param_suppressed1(_: u64) {
    }

    fun unused_param_suppressed2(_x: u64) {
    }

    native fun unused_native_ok(x: u64, y: bool);
}

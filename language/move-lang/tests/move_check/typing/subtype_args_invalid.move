module 0x8675309::M {
    struct S {}

    fun mut<T>(x: &mut T) {}
    fun imm_mut<T>(x: &T, y: &mut T) {}
    fun mut_imm<T>(x: &mut T, y: &T) {}
    fun mut_mut<T>(x: &mut T, y: &mut T) {}

    fun t0() {
        mut<u64>(&0);
        mut<u64>(&S{});
    }

    fun t1() {
        imm_mut<u64>(&0, &0);
        mut_imm<u64>(&0, &0);
        mut_mut<u64>(&0, &0);
    }
}

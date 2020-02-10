module M {
    struct S {}

    fun imm<T>(x: &T) {}
    fun imm_mut<T>(x: &T, y: &mut T) {}
    fun mut_imm<T>(x: &mut T, y: &T) {}
    fun imm_imm<T>(x: &T, y: &T) {}

    fun t0() {
        imm(&mut 0);
        imm(&0);

        imm(&mut S{});
        imm(&S{});
    }

    fun t1() {
        imm_mut(&mut 0, &mut 0);
        mut_imm(&mut 0, &mut 0);
        imm_imm(&mut 0, &mut 0);
    }
}

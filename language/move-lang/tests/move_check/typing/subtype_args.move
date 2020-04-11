module M {
    struct S {}

    fun imm<T>(_x: &T) {}
    fun imm_mut<T>(_x: &T, _y: &mut T) {}
    fun mut_imm<T>(_x: &mut T, _y: &T) {}
    fun imm_imm<T>(_x: &T, _y: &T) {}

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

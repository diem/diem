module M {
    struct S {}

    imm<T>(x: &T) {}
    imm_mut<T>(x: &T, y: &mut T) {}
    mut_imm<T>(x: &mut T, y: &T) {}
    imm_imm<T>(x: &T, y: &T) {}

    t0() {
        imm(&mut 0);
        imm(&0);

        imm(&mut S{});
        imm(&S{});
    }

    t1() {
        imm_mut(&mut 0, &mut 0);
        mut_imm(&mut 0, &mut 0);
        imm_imm(&mut 0, &mut 0);
    }
}

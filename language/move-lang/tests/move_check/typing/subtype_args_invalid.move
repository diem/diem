module M {
    struct S {}

    mut<T>(x: &mut T) {}
    imm_mut<T>(x: &T, y: &mut T) {}
    mut_imm<T>(x: &mut T, y: &T) {}
    mut_mut<T>(x: &mut T, y: &mut T) {}

    t0() {
        mut<u64>(&0);
        mut<u64>(&S{});
    }

    t1() {
        imm_mut<u64>(&0, &0);
        mut_imm<u64>(&0, &0);
        mut_mut<u64>(&0, &0);
    }
}

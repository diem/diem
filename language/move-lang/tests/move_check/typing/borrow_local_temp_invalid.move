module M {
    t0() {
        &();
        &(0, 1);
        &(0, 1, true, 0x0);
    }

    t1() {
        &(&0);
        &(&mut 1);
        &mut &2;
        &mut &mut 3;
    }
}

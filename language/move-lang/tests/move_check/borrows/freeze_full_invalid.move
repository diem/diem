module M {
    struct S { f: u64, g: u64 }
    id<T>(r: &T): &T {
        r
    }
    id_mut<T>(r: &mut T): &mut T {
        r
    }

    t0() {
        let x = &mut 0;
        freeze(x);
        *x = 0;

        let x = id_mut(&mut 0);
        freeze(x);
        *x = 0;
    }

}

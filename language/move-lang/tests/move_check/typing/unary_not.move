module M {
    resource struct R {
        f: bool
    }

    t0(x: bool, r: R) {
        !true;
        !false;
        !x;
        !copy x;
        !move x;
        !r.f;
        let R {f} = r;
    }
}

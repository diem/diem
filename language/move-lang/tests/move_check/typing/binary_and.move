module M {
    resource struct R {
        f: bool
    }

    t0(x: bool, r: R) {
        true && false;
        false && true;
        true && false;
        (true) && (true);
        copy x && move x;
        r.f && r.f;
        true && false && (true && false);
        let R {f} = r;
    }
}

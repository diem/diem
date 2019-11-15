module M {
    resource struct R {
        f: bool
    }

    t0(x: bool, r: R) {
        !&true;
        !&false;
        !0;
        !1;
        !r;
        !r;
        !(0, false);
        !();
    }
}

module M {
    resource struct R {
        f: bool
    }

    t0(x: bool, r: R) {
        -&0;
        -&1;
        -true;
        -false;
        -r;
        -r;
        -(0, false);
        -();
    }
}

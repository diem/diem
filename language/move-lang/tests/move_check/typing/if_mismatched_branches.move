module M {
    t0(cond: bool) {
        if (cond) () else 0;
        if (cond) 0 else ();
    }

    t1(cond: bool) {
        if (cond) 0x0 else 0;
        if (cond) 0 else false;
    }

    t2(cond: bool) {
        if (cond) (0, false) else (1, 1);
        if (cond) (0, false) else (false, false);
        if (cond) (0, false) else (true, 0x0);
    }

    t3(cond: bool) {
        if (cond) (0, false, 0) else (0, false);
        if (cond) (0, false) else (0, false, 0);
    }

}

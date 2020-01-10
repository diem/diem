module M {
    struct S {}

    tmove() {
        let x = 0;
        move x;
        let _ = move x + 1;

        let s = S{};
        let _s2 = s;
        let _s3 = s;
    }

    tcopy() {
        let x = 0;
        move x;
        let _ = x + 1;

        let s = S{};
        let _s2 = s;
        let _s3 = copy s;
    }

    tborrow() {
        let x = 0;
        move x;
        let _ = &x;

        let s = S{};
        let _s2 = s;
        let _s3 = &s;
    }

}

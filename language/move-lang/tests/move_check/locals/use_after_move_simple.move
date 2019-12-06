module M {
    struct S {}

    tmove() {
        let x = 0;
        move x;
        let y = move x + 1;

        let s = S{};
        let s2 = s;
        let s3 = s;
    }

    tcopy() {
        let x = 0;
        move x;
        let y = x + 1;

        let s = S{};
        let s2 = s;
        let s3 = copy s;
    }

    tborrow() {
        let x = 0;
        move x;
        let y = &x;

        let s = S{};
        let s2 = s;
        let s3 = &s;
    }

}

module 0x42::TestFriendError {

    struct R {
        x: u64,
    }

    public fun f() {}

    spec f {
        pragma friend = 0x1::M::some_other_fun;
    }

    public fun g() {}

    spec g {
        pragma friend = h;
        pragma opaque; // Errors here since g can't be opaque with a friend
    }

    public fun h() {
        f(); // Errors here since f can only be called from M::some_other_fun
        g();
    }

    spec h {
        pragma friend = i;
    }

    public fun i() {
        g(); // Errors here since g can only be called from h
    }
}

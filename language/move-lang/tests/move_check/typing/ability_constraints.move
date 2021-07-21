address 0x42 {
module M {
    struct S has copy, drop, store, key {}
    struct R has key, store {}
    struct Box<T> has copy, drop, store, key { f: T }
    struct Pair<T1, T2> has copy, drop, store, key { f1: T1, f2: T2 }

    fun c<T: copy>() {}
    fun d<T: drop>() {}
    fun s<T: store>() {}
    fun k<T: key>() {}
    fun sk<T: store + key>() {}
    fun cds<T: copy + drop + store>() {}

    struct Sc<phantom T: copy> {}
    struct Sd<phantom T: drop> {}
    struct Ss<phantom T: store> {}
    struct Sk<phantom T: key> {}
    struct Ssk<phantom T: store + key> {}
    struct Scds<phantom T: copy + drop + store> {}

    // tests that a variety of constraint instantiations are all valid
    fun t1<
        Tc: copy,
        Td: drop,
        Ts: store,
        Tk: key,
        Tcd: copy + drop,
        Tcs: copy + store,
        Tds: drop + store,
        Tsk: store + key,
        Tcds: copy + drop + store
    >() {
        // functions with prims
        c<u64>();
        d<signer>();
        s<bool>();
        cds<address>();
        c<vector<u64>>();
        d<vector<signer>>();
        s<vector<bool>>();
        cds<vector<address>>();
        // structs with prims
        let Sc {} = Sc<u64> {};
        let Sd {} = Sd<signer> {};
        let Ss {} = Ss<bool> {};
        let Scds {} = Scds<address> {};
        let Sc {} = Sc<vector<u64>> {};
        let Sd {} = Sd<vector<signer>> {};
        let Ss {} = Ss<vector<bool>> {};
        let Scds {} = Scds<vector<address>> {};

        // functions with tparams
        c<Tc>();
        c<Tcd>();
        c<Tcs>();
        c<Tcds>();
        d<Td>();
        d<Tcd>();
        d<Tds>();
        d<Tcds>();
        s<Ts>();
        s<Tcs>();
        s<Tds>();
        s<Tsk>();
        s<Tcds>();
        k<Tk>();
        k<Tsk>();
        sk<Tsk>();
        cds<Tcds>();
        // structs with tparams
        let Sc {} = Sc<Tc> {};
        let Sc {} = Sc<Tcd> {};
        let Sc {} = Sc<Tcs> {};
        let Sc {} = Sc<Tcds> {};
        let Sd {} = Sd<Td> {};
        let Sd {} = Sd<Tcd> {};
        let Sd {} = Sd<Tds> {};
        let Sd {} = Sd<Tcds> {};
        let Ss {} = Ss<Ts> {};
        let Ss {} = Ss<Tcs> {};
        let Ss {} = Ss<Tds> {};
        let Ss {} = Ss<Tsk> {};
        let Ss {} = Ss<Tcds> {};
        let Sk {} = Sk<Tk> {};
        let Sk {} = Sk<Tsk> {};
        let Ssk {} = Ssk<Tsk> {};
        let Scds {} = Scds<Tcds> {};

        // functions with structs
        c<S>();
        c<Box<S>>();
        c<Pair<Box<S>, S>>();
        d<S>();
        d<Box<S>>();
        d<Pair<Box<S>, S>>();
        s<S>();
        s<R>();
        s<Box<S>>();
        s<Box<R>>();
        s<Pair<Box<S>, S>>();
        s<Pair<R, Box<Pair<R, S>>>>();
        k<R>();
        k<Box<R>>();
        k<Pair<R, Box<R>>>();
        k<Pair<R, Box<Pair<R, S>>>>();
        sk<R>();
        sk<Box<R>>();
        sk<Pair<R, Box<R>>>();
        sk<Pair<R, Box<Pair<R, S>>>>();
        cds<S>();
        cds<Box<S>>();
        cds<Pair<Box<S>, S>>();
        // structs with structs
        let Sc {} = Sc<S> {};
        let Sc {} = Sc<Box<S>> {};
        let Sc {} = Sc<Pair<Box<S>, S>> {};
        let Sd {} = Sd<S> {};
        let Sd {} = Sd<Box<S>> {};
        let Sd {} = Sd<Pair<Box<S>, S>> {};
        let Ss {} = Ss<S> {};
        let Ss {} = Ss<R> {};
        let Ss {} = Ss<Box<S>> {};
        let Ss {} = Ss<Box<R>> {};
        let Ss {} = Ss<Pair<Box<S>, S>> {};
        let Ss {} = Ss<Pair<R, Box<Pair<R, S>>>> {};
        let Sk {} = Sk<R> {};
        let Sk {} = Sk<Box<R>> {};
        let Sk {} = Sk<Pair<R, Box<R>>> {};
        let Sk {} = Sk<Pair<R, Box<Pair<R, S>>>> {};
        let Ssk {} = Ssk<R> {};
        let Ssk {} = Ssk<Box<R>> {};
        let Ssk {} = Ssk<Pair<R, Box<R>>> {};
        let Ssk {} = Ssk<Pair<R, Box<Pair<R, S>>>> {};
        let Scds {} = Scds<S> {};
        let Scds {} = Scds<Box<S>> {};
        let Scds {} = Scds<Pair<Box<S>, S>> {};

    }
}
}

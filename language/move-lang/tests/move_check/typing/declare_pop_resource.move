module M {
    resource struct R {f: u64}

    fun t0() {
        let _: R;
        let _r: R;
        let (_, _):(R, R);
    }
}

module M {
    resource struct R {}
    struct Cup<T> {}

    fun t0(x8: u8, x64: u64, x128: u128) {
        (false as u8);
        (true as u128);

        (() as u64);
        ((0, 1) as u8);

        (0 as bool);
        (0 as address);
        R{} = (0 as R);
        (0 as Cup<u8>);
        (0 as ());
        (0 as (u64, u8));

	(x"1234" as u64);
    }
}

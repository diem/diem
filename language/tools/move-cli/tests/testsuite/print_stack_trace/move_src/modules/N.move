address 0x2 {
module N {
    use 0x2::M;

    public fun foo<T1, T2>(): u64 {
        let x = 3;
        let y = &mut x;
        let z = M::sum(4);
        _ = y;
        z
    }
}
}

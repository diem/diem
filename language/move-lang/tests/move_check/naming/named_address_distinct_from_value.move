// These checks straddle a few different passes but
// Named addresses are distinct from their value

module A::M {
    const C: u64 = 0;
    struct S {}
    public fun s(): S { S{} }
}

module A::Ex {
    use 0x42::M;
    friend 0x42::M;
    public fun ex(): 0x42::M::S {
        0x42::M::C;
        0x42::M::s()
    }
}

// These checks straddle a few different passes but
// Named addresses are distinct from each other, even if the value is the same

address A = 0x42;
address B = 0x42;

module A::M {
    const C: u64 = 0;
    struct S {}
    public fun s(): S { S{} }
}

module A::Ex {
    use B::M;
    friend B::M;
    public fun ex(): B::M::S {
        B::M::C;
        B::M::s()
    }
}

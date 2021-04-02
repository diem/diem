module 0x42::M {
    // visibility modifiers on constants fail during parsing
    public const Foo: u64 = 0;
}

module 0x42::M {
    // native modifiers on constants fail during parsing
    native const Foo: u64 = 0;
}

// Duplicate modules need to be checked with respect to name=>value mapping

// Both modules named
module A::M {}
module B::M {}

// Anon, named
module 0x1::M {}
module M::M {}

// Named, Anon
module K::M {}
module 0x19::M {}

// Duplicate modules need to be checked with respect to name=>value mapping

// Both modules named
address OneA = 0x1;
address OneB = 0x1;
module OneA::M {}
module OneB::M {}

// Anon, named
address TwoB = 0x2;
module 0x2::M {}
module TwoB::M {}

// Named, Anon
address ThreeA = 0x3;
module ThreeA::M {}
module 0x3::M {}

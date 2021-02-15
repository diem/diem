address 0x2 {
module M {}
}

address 0x3 {
module M {
    friend 0x2::M;
}
}

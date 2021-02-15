address 0x42 {
module M {
    friend Self;
}
}

address 0x43 {
module M {
    friend 0x43::M;
}
}

address 0x42 {
module M {
    const C1: &u64 = &0;
    const C2: &mut u64 = &0;
    const C3: () = ();
    const C4: (address, bool) = (@0x0, false);
}
}

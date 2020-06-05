address 0x42 {
module M {
    const C: u64 = {
        ();
        0;
        { (); () };
        !false;
        false && false;
        true && true;
        2 + 1;
        2 - 1;
        2 * 1;
        2 / 1;
        2 % 1;
        2 >> 1;
        2 << 1;
        2 ^ 1;
        2 & 1;
        2 | 1;
        0x0 == 0x1;
        b"ab" != x"01";
        (0 as u8);
        (0 as u64);
        (0 as u128);
        (0: u8);
        0
    };
}
}

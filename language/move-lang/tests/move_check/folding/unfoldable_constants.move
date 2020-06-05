address 0x42 {
module M {
    const SHL0: u8 = 1 << 8;
    const SHL1: u64 = 1 << 64;
    const SHL2: u128 = 1 << 128;

    const SHR0: u8 = 0 >> 8;
    const SHR1: u64 = 0 >> 64;
    const SHR2: u128 = 0 >> 128;

    const DIV0: u8 = 1 / 0;
    const DIV1: u64 = 1 / 0;
    const DIV2: u128 = 1 / 0;

    const MOD0: u8 = 1 % 0;
    const MOD1: u64 = 1 % 0;
    const MOD2: u128 = 1 % 0;

    const ADD0: u8 = 255 + 255;
    const ADD1: u64 = 18446744073709551615 + 18446744073709551615;
    const ADD2: u128 =
        340282366920938463463374607431768211450 + 340282366920938463463374607431768211450;

    const SUB0: u8 = 0 - 1;
    const SUB1: u64 = 0 - 1;
    const SUB2: u128 = 0 - 1;

    const CAST0: u8 = ((256: u64) as u8);
    const CAST1: u64 = ((340282366920938463463374607431768211450: u128) as u64);
}
}

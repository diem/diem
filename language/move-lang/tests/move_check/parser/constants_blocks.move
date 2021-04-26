address 0x42 {
module M {
    fun u(): u64 { 0 }

    const C1: u64 = { u() };
    const C2: u64 = { 0 + 1 * 2 % 3 / 4 >> 1 << 2 };
    const C3: bool = { loop () };
    const C4: u8 = { if (false) 0 else 1 };
    const C5: vector<vector<bool>> = { abort 0 };
    const C6: u128 = { 0 };
    const C7: () = {
        let x = 0;
        let y = 1;
        x + y;
    };
    const C8: address = {
        0;
        1 + 1;
        u();
        @0x0
    };
}
}

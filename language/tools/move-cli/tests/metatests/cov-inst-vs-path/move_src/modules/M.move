address 0x42 {
module M {
    public fun test(x: u8, y: u8): u8  {
        let a = if (x > 0) {
            y + 1
        } else {
            y
        };

        let b = if (y > 0) {
            x + 1
        } else {
            x
        };

        a + b
    }
}
}

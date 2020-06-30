script {
    const C: u64 = 0;
    const BYTES: vector<u8> = b"hello";

    fun check() {
        assert(C == 0, 42);
        assert(BYTES == b"hello", 42);

        // little weird we can do this... might want to warn?
        assert(*&C == 0, 42);
        assert(*&BYTES == b"hello", 42);

        let c = &mut C;
        *c = 1;
        let b = &mut BYTES;
        *b = b"bye";
        assert(*c == 1, 42);
        assert(*b == b"bye", 42);

        // Borrows the local copy, not the constant. No update
        assert(C == 0, 42);
        assert(BYTES == b"hello", 42);
    }
}

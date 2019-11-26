// Addresses are at most 32 bytes long.
// FIXME: The lalrpop parser does not support a custom error message with a location
// and the Move compiler's normal error handling requires a source location, so this
// currently leads to a panic and a test failure.
// address 0x112233445566778899101122334455667788992011223344556677889930112233:

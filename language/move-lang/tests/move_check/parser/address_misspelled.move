// FIXME: The lalrpop parser does not support a custom error message with a location
// and the Move compiler's normal error handling requires a source location, so this
// currently leads to a panic and a test failure.
// addrexx 0x1:

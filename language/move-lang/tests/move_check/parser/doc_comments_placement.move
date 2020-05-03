/// This is documentation for module M.

/// Documentation comments can have multiple blocks.
/** They may use different limiters. */
module M {
    /** There can be no doc comment on uses. */
    use 0x0::Transaction;

    /// This is f.
    fun f() { }

    /// This is T
    struct T {
        /// This is a field of T.
        f: u64,
        /// There can be no doc comment after last field.
    }

    /// This is some spec.
    spec module {
        /// This is a pragma.
        pragma verify = true;
        /// There can be no doc comment after last block member.
    }

    /// There can be no doc comment after last module item.
}

/// There can be no doc comment at end of file.

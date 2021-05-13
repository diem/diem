/// Utilities for comparing Move values based on their representation in BCS.
module Std::Compare {
    use Std::Vector;

    // Move does not have signed integers, so we cannot use the usual 0, -1, 1 convention to
    // represent EQUAL, LESS_THAN, and GREATER_THAN. Instead, we fun a new convention using u8
    // constants:
    const EQUAL: u8 = 0;
    const LESS_THAN: u8 = 1;
    const GREATER_THAN: u8 = 2;

    /// Compare vectors `v1` and `v2` using (1) vector contents from right to left and then
    /// (2) vector length to break ties.
    /// Returns either `EQUAL` (0u8), `LESS_THAN` (1u8), or `GREATER_THAN` (2u8).
    ///
    /// This function is designed to compare BCS (Binary Canonical Serialization)-encoded values
    /// (i.e., vectors produced by `BCS::to_bytes`). A typical client will call
    /// `Compare::cmp_bcs_bytes(BCS::to_bytes(&t1), BCS::to_bytes(&t2))`. The comparison provides the
    /// following guarantees w.r.t the original values t1 and t2:
    /// - `cmp_bcs_bytes(bcs(t1), bcs(t2)) == LESS_THAN` iff `cmp_bcs_bytes(t2, t1) == GREATER_THAN`
    /// - `Compare::cmp<T>(t1, t2) == EQUAL` iff `t1 == t2` and (similarly)
    ///   `Compare::cmp<T>(t1, t2) != EQUAL` iff `t1 != t2`, where `==` and `!=` denote the Move
    ///    bytecode operations for polymorphic equality.
    /// - for all primitive types `T` with `<` and `>` comparison operators exposed in Move bytecode
    ///   (`u8`, `u64`, `u128`), we have
    ///   `compare_bcs_bytes(bcs(t1), bcs(t2)) == LESS_THAN` iff `t1 < t2` and (similarly)
    ///   `compare_bcs_bytes(bcs(t1), bcs(t2)) == LESS_THAN` iff `t1 > t2`.
    ///
    /// For all other types, the order is whatever the BCS encoding of the type and the comparison
    /// strategy above gives you. One case where the order might be surprising is the `address`
    /// type.
    /// CoreAddresses are 16 byte hex values that BCS encodes with the identity function. The right
    /// to left, byte-by-byte comparison means that (for example)
    /// `compare_bcs_bytes(bcs(0x01), bcs(0x10)) == LESS_THAN` (as you'd expect), but
    /// `compare_bcs_bytes(bcs(0x100), bcs(0x001)) == LESS_THAN` (as you probably wouldn't expect).
    /// Keep this in mind when using this function to compare addresses.
    ///
    /// > TODO: there is currently no specification for this function, which causes no problem because it is not yet
    /// > used in the Diem framework. However, should this functionality be needed in specification, a customized
    /// > native abstraction is needed in the prover framework.
    public fun cmp_bcs_bytes(v1: &vector<u8>, v2: &vector<u8>): u8 {
        let i1 = Vector::length(v1);
        let i2 = Vector::length(v2);
        let len_cmp = cmp_u64(i1, i2);

        // BCS uses little endian encoding for all integer types, so we choose to compare from left
        // to right. Going right to left would make the behavior of Compare.cmp diverge from the
        // bytecode operators < and > on integer values (which would be confusing).
        while (i1 > 0 && i2 > 0) {
            i1 = i1 - 1;
            i2 = i2 - 1;
            let elem_cmp = cmp_u8(*Vector::borrow(v1, i1), *Vector::borrow(v2, i2));
            if (elem_cmp != 0) return elem_cmp
            // else, compare next element
        };
        // all compared elements equal; use length comparion to break the tie
        len_cmp
    }

    /// Compare two `u8`'s
    fun cmp_u8(i1: u8, i2: u8): u8 {
        if (i1 == i2) EQUAL
        else if (i1 < i2) LESS_THAN
        else GREATER_THAN
    }

    /// Compare two `u64`'s
    fun cmp_u64(i1: u64, i2: u64): u8 {
        if (i1 == i2) EQUAL
        else if (i1 < i2) LESS_THAN
        else GREATER_THAN
    }

}

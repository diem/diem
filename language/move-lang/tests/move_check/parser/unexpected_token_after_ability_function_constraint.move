address 0x42 {
module M {
    // incorrect delim for ability constraint
    fun foo<T: copy & drop>() {}
}
}

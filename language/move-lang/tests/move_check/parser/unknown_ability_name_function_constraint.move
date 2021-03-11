address 0x42 {
module M {
    // invalid constraint
    fun foo<T: blah>() {}
}
}

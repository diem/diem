address 0x42 {
module M {
    // unexpected trailing plus
    fun foo<T: copy +>() {}
}
}

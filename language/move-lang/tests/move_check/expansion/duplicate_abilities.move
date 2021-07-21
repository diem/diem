address 0x42 {
module M {
    // invalid duplicate abilities
    struct Foo has copy, copy {}
    struct Bar<T: drop + drop> { f: T }
    fun baz<T: store + store>() {}
}
}
script {
    fun main<T: key + key>() {}
}

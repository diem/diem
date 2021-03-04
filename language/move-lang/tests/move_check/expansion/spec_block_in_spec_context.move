address 0x42 {
module M {
    spec module {
        define S(): () {
            spec {};
        }
    }

    fun t(): () {
        spec {};
    }

    fun a() {}
    fun z() {}
}
}

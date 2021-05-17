address 0x42 {
module M {
    spec module {
        fun S(): () {
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

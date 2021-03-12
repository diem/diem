module 0x8675309::M {
    struct no {}
    struct X { f: no }

    fun mk(x: no): no {
        no {}
    }

    struct no2 {}
    struct Y { f: no }

    fun mk2(x: no2): no2 {
        let no2 {} = x;
        no2 {}
    }

}

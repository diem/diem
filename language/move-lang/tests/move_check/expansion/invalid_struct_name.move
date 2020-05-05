module M {
    struct no {}
    struct X { f: no }

    fun mk(x: no): no {
        no {}
    }

    resource struct no2 {}
    resource struct Y { f: no }

    fun mk2(x: no2): no2 {
        let no2 {} = x;
        no2 {}
    }

}

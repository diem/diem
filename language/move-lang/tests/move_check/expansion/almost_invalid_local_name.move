module 0x8675309::M {
    struct F has drop { No: u64 }

    fun t(_No: u64) {
    }

    fun t2() {
        let _No = 100;
    }

    fun t3() {
        let _No;
        _No = 100;
    }

    fun t4() {
        let _No = 100;
        F { No: _No };
    }

}

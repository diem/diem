module 0x8675309::M {
    struct R {}

    public fun reassign_parameter(r: R) {
        let R { } = r;
        r = R {};
        if  (true) {
            let R { } = r;
        }
    }

}

module M {
    resource struct R {}

    public fun reassign_parameter(r: R) {
        let R { } = r;
        r = R {};
        if  (true) {
            let R { } = r
        }
    }

}

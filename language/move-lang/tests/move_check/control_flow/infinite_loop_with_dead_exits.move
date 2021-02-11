address 0x42 {
module M {

    // fun t1() {
    //     loop {
    //         continue;
    //         break
    //     };
    // }

    fun t2() {
        loop {
            if (return) break;
        }
    }

}
}

script {
    use 0x42::M1;
    use 0x42::M2;

    fun main(x: u8) {
        if (x == 0) {
            M1::test();
        } else {
            M2::test();
        }
    }
}

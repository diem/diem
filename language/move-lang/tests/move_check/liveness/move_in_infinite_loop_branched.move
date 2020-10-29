script {
    fun main(cond: bool) {
        let x = 0;
        if (cond) {
            loop { let y = move x; y / y; }
        } else {
            loop { let y = move x; y % y; }
        }
    }
}

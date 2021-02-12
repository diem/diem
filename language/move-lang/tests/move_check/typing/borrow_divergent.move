script {
    fun main1() {
        loop {
           &break;
        }
    }
}

script {
    fun main2() {
        &{ return };
    }
}


script {
    fun main3(cond: bool) {
        &(if (cond) return else return);
    }
}

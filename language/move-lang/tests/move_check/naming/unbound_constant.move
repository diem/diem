address 0x42 {
module M {
    fun t() {
        let x = CONSTANT;
        let y = Self::CONSTANT;
        0 + CONSTANT + Self::CONSTANT;
    }
}
}

script {
    fun t() {
        let x = CONSTANT;
        let y = Self::CONSTANT;
        0 + CONSTANT + Self::CONSTANT;
    }
}

script {
    use 0x1::Vector;
    fun main() {
        let v = Vector::empty<bool>();
        let _ref = Vector::borrow(&v, 0);
    }
}

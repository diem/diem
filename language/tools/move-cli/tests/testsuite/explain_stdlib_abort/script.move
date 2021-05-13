script {
    use Std::Vector;
    fun main() {
        let v = Vector::empty<bool>();
        let _ref = Vector::borrow(&v, 0);
    }
}

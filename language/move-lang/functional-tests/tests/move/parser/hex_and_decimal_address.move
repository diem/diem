script {
    fun main() {
        assert(@0 == @0x0, 42);
        assert(@0xF == @15, 42);
        assert(@0x42 == @66, 42);
    }
}

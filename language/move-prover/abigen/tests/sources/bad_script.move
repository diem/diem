script {
    fun bad_script() {
        abort true  // type error, abort code must be a u64
    }
}

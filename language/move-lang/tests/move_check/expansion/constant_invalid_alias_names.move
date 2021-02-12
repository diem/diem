address 0x42 {

module N {
    const C: bool = false;
}

module M {
    use 0x42::N::{C as c1, C as _C1, C as Self};
}
}

script {
    use 0x42::N::{C as c1, C as _C1, C as Self};
    fun main() {}
}

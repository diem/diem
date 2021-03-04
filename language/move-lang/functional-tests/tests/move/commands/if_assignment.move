address {{default}} {
module ReassignCond {
    public fun reassign_cond(a: address, b: bool): address {
        if (b) {
            a = 0x2;
        };
        a
    }
}
}

//! new-transaction

script {
    use {{default}}::ReassignCond::reassign_cond;
    fun main() {
        assert(reassign_cond(0x1, false) == 0x1, 42);
    }
}

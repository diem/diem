module 0x42::ReachingDefTest {

    struct R has key {
        x: u64,
        y: bool
    }

	fun basic(a: u64, b: u64): u64 {
	    let x = (a + b) / a;
	    x + 1
	}

    fun create_resource(sender: &signer) {
        let r = R{ x: 1, y: false};
        move_to<R>(sender, r);
    }
}

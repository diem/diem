module ReachingDefTest {

    resource struct R {
        x: u64,
        y: bool
    }

	fun basic(a: u64, b: u64): u64 {
	    let x = (a + b) / a;
	    x + 1
	}

    fun create_resource() {
        let r = R{ x: 1, y: false};
        move_to_sender<R>(r);
    }
}

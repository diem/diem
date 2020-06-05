module TestBranching {
    fun branching(cond: bool) : u64 {
      let x = if (cond) { 3 } else { 4 };
      x
    }
}

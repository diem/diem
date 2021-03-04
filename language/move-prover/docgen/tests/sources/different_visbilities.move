address 0x2 {
module TestViz {
    /// This is a public function
    public fun this_is_a_public_fun() { }

    // /// This is a public friend function
    // public(friend) fun this_is_a_public_friend_fun() {}

    /// This is a public script function
    public(script) fun this_is_a_public_script_fun() {}

    /// This is a private function
    fun this_is_a_private_fun() {}
}
}

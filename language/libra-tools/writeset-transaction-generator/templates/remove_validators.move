script {
    use 0x1::LibraSystem;
    fun main(libra_root: &signer) {
        {{#each addresses}}
        LibraSystem::remove_validator(libra_root, 0x{{this}});
        {{/each}}
    }
}

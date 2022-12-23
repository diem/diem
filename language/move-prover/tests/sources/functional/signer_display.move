module 0x42::SignerDisplay {
    use Std::Signer;

    // Test the signer value display in the error trace
    fun f_incorrect(account: &signer) {
        spec {
            assert Signer::spec_address_of(account) == @0x1;
        }
    }
}

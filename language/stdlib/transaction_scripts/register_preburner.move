use 0x0::LBR;
use 0x0::Libra;

// Publish a newly created `Preburn` resource under the sender's account.
// This will fail if the sender already has a published `Preburn` resource.
fun main() {
    Libra::publish_preburn(Libra::new_preburn<LBR::T>())
}

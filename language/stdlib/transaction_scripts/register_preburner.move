use 0x0::Libra;

// Publish a newly created `Preburn<Token>` resource under the sender's account.
// This will fail if the sender already has a published `Preburn<Token>` resource.
fun main<Token>() {
    Libra::publish_preburn(Libra::new_preburn<Token>())
}

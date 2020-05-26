script {
use 0x0::Libra;
fun main<Currency>(account: &signer, allow_minting: bool) {
    Libra::update_minting_ability<Currency>(account, allow_minting)
}
}

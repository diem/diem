script {
use 0x1::Libra;
fun main<Currency>(account: &signer, allow_minting: bool) {
    Libra::update_minting_ability<Currency>(account, allow_minting)
}
}

use 0x0::Libra;
fun main<Currency>(allow_minting: bool) {
    Libra::update_minting_ability<Currency>(allow_minting)
}

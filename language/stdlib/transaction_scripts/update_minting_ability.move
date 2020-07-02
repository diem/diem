script {
use 0x1::Libra;

/// Allows--true--or disallows--false--minting of `currency` based upon `allow_minting`.
fun update_minting_ability<Currency>(
    tc_account: &signer,
    allow_minting: bool
    ) {
    Libra::update_minting_ability<Currency>(tc_account, allow_minting);
}
}

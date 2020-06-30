script {
use 0x1::Libra;
use 0x1::Roles::{Self, TreasuryComplianceRole};

/// Allows--true--or disallows--false--minting of `currency` based upon `allow_minting`.
fun update_minting_ability<Currency>(account: &signer, allow_minting: bool) {
    let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
    Libra::update_minting_ability<Currency>(&tc_capability, allow_minting);
    Roles::restore_capability_to_privilege(account, tc_capability);
}
}

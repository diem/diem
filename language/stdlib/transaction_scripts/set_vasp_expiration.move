script {
use 0x1::VASP;
use 0x1::Roles::{Self, LibraRootRole};

/// Set the expiration time of the parent vasp account at `parent_vasp_address`
/// to expire at `new_expiration_date` specified in microseconds according to
/// blockchain time. This can only be invoked by the Libra root account.
fun set_vasp_expiration(
    association: &signer,
    parent_vasp_address: address,
    new_expiration_date: u64,
) {
    let assoc_root_capability = Roles::extract_privilege_to_capability<LibraRootRole>(association);
    VASP::set_vasp_expiration(&assoc_root_capability, parent_vasp_address, new_expiration_date);
    Roles::restore_capability_to_privilege(association, assoc_root_capability);
}
}

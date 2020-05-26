// Error codes:
// 7000 -> INSUFFICIENT_PRIVILEGES
// 7001 -> INVALID_ROOT_VASP_ACCOUNT
// 7002 -> INVALID_CHILD_VASP_ACCOUNT
// 7003 -> CHILD_ACCOUNT_STILL_PARENT
// 7004 -> INVALID_PUBLIC_KEY
address 0x0 {

module VASP {
    use 0x0::LibraTimestamp;
    use 0x0::Signer;
    use 0x0::Testnet;
    use 0x0::Transaction;
    use 0x0::Vector;
    use 0x0::Association;
    use 0x0::LibraConfig;
    use 0x0::VASPRegistry::{Self, VASPRegistrationCapability};

    // A ParentVASP is held only by the root VASP account and holds the
    // VASP-related metadata for the account. It is subject to a time
    // limitation, and needs to be re-certified by an association
    // account before the certification expires.
    struct ParentVASP {
        // The human readable name of this VASP.
        human_name: vector<u8>,
        // The base_url holds the URL to be used for off-chain
        // communication. This contains the whole URL (e.g. https://...).
        base_url: vector<u8>,
        // Expiration date in microseconds from unix epoch. Mutable, but only by Association
        expiration_date: u64,
        // 32 byte Ed25519 public key whose counterpart must be used to sign
        // (1) the payment metadata for on-chain travel rule transactions
        // (2) the KYC information exchanged in the off-chain travel rule protocol.
        // Note that this is different than `authentication_key` used in LibraAccount::T, which is
        // a hash of a public key + signature scheme identifier, not a public key. Mutable
        compliance_public_key: vector<u8>,
    }

    // A ChildVASP type for representing `AccountType<ChildVASP>` accounts.
    // The parent VASP can create as many of these as they wish, and them
    // being a VASP account is dependent on the continuing certification of
    // the parent VASP account by the association.
    struct ChildVASP { parent_vasp_addr: address }

    // A registration capability that allows this module to add and remove
    // vasps from the on-chain VASP registry.
    resource struct RegistrationCapability {
        cap: VASPRegistrationCapability,
    }

    public fun initialize(config_account: &signer) {
        let cap = VASPRegistry::initialize(config_account);
        move_to(config_account, RegistrationCapability{cap});
    }

    ///////////////////////////////////////////////////////////////////////////
    // Association called functions for parent VASP accounts
    ///////////////////////////////////////////////////////////////////////////

    // Renew's `parent_vasp`'s certification
    public fun recertify_vasp(parent_vasp: &mut ParentVASP) {
        parent_vasp.expiration_date = LibraTimestamp::now_microseconds() + cert_lifetime();
    }

    // Non-destructively decertify `parent_vasp`. Can be
    // recertified later on via `recertify_vasp`.
    public fun decertify_vasp(parent_vasp_addr: address, parent_vasp: &mut ParentVASP)
    acquires RegistrationCapability {
        // Expire the parent credential.
        parent_vasp.expiration_date = 0;
        delist_vasp(parent_vasp_addr)
    }

    public fun delist_vasp(parent_vasp_addr: address)
    acquires RegistrationCapability {
        Association::assert_sender_is_association();
        let cap = borrow_global<RegistrationCapability>(LibraConfig::default_config_address());
        VASPRegistry::remove_vasp(parent_vasp_addr, &cap.cap);
    }

    public fun register_vasp(parent_vasp_addr: address)
    acquires RegistrationCapability {
        Association::assert_sender_is_association();
        let cap = borrow_global<RegistrationCapability>(LibraConfig::default_config_address());
        VASPRegistry::add_vasp(parent_vasp_addr, &cap.cap);
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be parent-vasp called functions
    ///////////////////////////////////////////////////////////////////////////

    public fun create_parent_vasp_credential(
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>
    ): ParentVASP {
        // NOTE: Only callable in testnet
        Transaction::assert(Testnet::is_testnet(), 10041);
        ParentVASP {
           // For testnet, so it should never expire. So set to u64::MAX
           expiration_date: 18446744073709551615,
           human_name,
           base_url,
           compliance_public_key,
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs
    ///////////////////////////////////////////////////////////////////////////

    // Create a child VASP resource for the sender
    // Aborts if the sender is not a ParentVASP
    public fun create_child_vasp(sender: &signer): ChildVASP {
        ChildVASP { parent_vasp_addr: Signer::address_of(sender) }
    }

    public fun child_parent_address(child: &ChildVASP): address {
        child.parent_vasp_addr
    }

    // Return true if `sender` is the parent VASP of `ChildVASP`
    public fun is_parent_vasp(child_vasp: &ChildVASP, sender: &signer): bool {
        Signer::address_of(sender) == child_vasp.parent_vasp_addr
    }

    // Return the human-readable name for the VASP account
    public fun human_name(parent_vasp: &ParentVASP): vector<u8> {
        *&parent_vasp.human_name
    }

    // Return the base URL for the VASP account
    public fun base_url(parent_vasp: &ParentVASP): vector<u8> {
        *&parent_vasp.base_url
    }

    // Return the compliance public key for the VASP account
    public fun compliance_public_key(parent_vasp: &ParentVASP): vector<u8> {
        *&parent_vasp.compliance_public_key
    }

    /// Rotate the compliance public key for `parent_vasp` to `new_key`
    public fun rotate_compliance_public_key(parent_vasp: &mut ParentVASP, new_key: vector<u8>) {
        Transaction::assert(Vector::length(&new_key) == 32, 7004);
        parent_vasp.compliance_public_key = new_key;
    }

    // Return the expiration date for the VASP account
    public fun expiration_date(parent_vasp: &ParentVASP): u64 {
        parent_vasp.expiration_date
    }

    fun parent_credential_expired(parent_credential: &ParentVASP): bool {
        parent_credential.expiration_date < LibraTimestamp::now_microseconds()
    }

    fun singleton_addr(): address {
        0xA550C18
    }

    // A year in microseconds
    fun cert_lifetime(): u64 {
        31540000000000
    }
}

}

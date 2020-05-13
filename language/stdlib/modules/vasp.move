// Error codes:
// 7000 -> INSUFFICIENT_PRIVILEGES
// 7001 -> INVALID_ROOT_VASP_ACCOUNT
// 7002 -> INVALID_CHILD_VASP_ACCOUNT
// 7003 -> CHILD_ACCOUNT_STILL_PARENT
// 7004 -> INVALID_PUBLIC_KEY
address 0x0 {

module VASP {
    use 0x0::AccountType;
    use 0x0::Association;
    use 0x0::LibraTimestamp;
    use 0x0::Testnet;
    use 0x0::Transaction;
    use 0x0::Vector;

    // A RootVASP is held only by the root VASP account and holds the
    // VASP-related metadata for the account. It is subject to a time
    // limitation, and needs to be re-certified by an association
    // account before the certification expires.
    struct RootVASP {
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
    // The root VASP can create as many of these as they wish, and them
    // being a VASP account is dependent on the continuing certification of
    // the root VASP account by the association.
    struct ChildVASP { is_certified: bool }

    // Association privilege needed to create root VASP accounts.
    struct CreationPrivilege { }

    // Initialize the VASP association singleton resource. Must be called
    // before  any VASPs are allowed in the system.
    public fun initialize() {
        Association::assert_sender_is_association();
        let sender = Transaction::sender();
        Transaction::assert(sender == singleton_addr(), 7000);
        AccountType::register<ChildVASP>();
        AccountType::register<RootVASP>();
        // Now publish and certify this account to allow granting of root
        // VASP accounts. We can perform all of these operations at once
        // since the singleton_addr() == sender, and singleton_addr() must
        // be an association address.
        AccountType::apply_for_granting_capability<RootVASP>();
        AccountType::certify_granting_capability<RootVASP>(sender);
        // Apply and certify that this account can transition Empty::T => RootVASP
        AccountType::apply_for_transition_capability<RootVASP>(sender);
        AccountType::grant_transition_capability<RootVASP>(sender);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Association called functions for root VASP accounts
    ///////////////////////////////////////////////////////////////////////////

    // Recertifies that the account at `addr` is the root account for a VASP.
    public fun recertify_vasp(addr: address) {
        // Verify that the sender is a correctly privileged association account
        assert_sender_is_assoc_vasp_privileged();
        // Verify that the account in question is still a valid root VASP account
        // But, it's cert might have expired. That's why we don't use
        // `is_root_vasp` here, but instead make sure `addr` is a RootVASP
        // account type.
        Transaction::assert(AccountType::is_a<RootVASP>(addr), 7001);
        let root_vasp = AccountType::account_metadata<RootVASP>(addr);
        root_vasp.expiration_date = LibraTimestamp::now_microseconds() + cert_lifetime();
        // The sending account must have a TransitionCapability<RootVASP>.
        AccountType::update<RootVASP>(addr, root_vasp);
    }

    // A certification of a VASP with its root account at `addr` can only
    // happen if there is an uncertified RootVASP account type published
    // under the address.
    public fun grant_vasp(addr: address) {
        assert_sender_is_assoc_vasp_privileged();
        // The sending account must have a TransitionCapability<RootVASP> capability
        AccountType::transition<RootVASP>(addr);
        AccountType::certify_granting_capability<ChildVASP>(addr);
    }

    // Non-destructively decertify the root vasp at `addr`. Can be
    // recertified later on via `recertify_vasp`.
    public fun decertify_vasp(addr: address) {
        assert_sender_is_assoc_vasp_privileged();
        Transaction::assert(is_root_vasp(addr), 7001);
        let root_vasp = AccountType::account_metadata<RootVASP>(addr);
        // Expire the root credential.
        root_vasp.expiration_date = 0;
        // Updat the root vasp metadata for the account with the new root vasp
        // credential.
        AccountType::update<RootVASP>(addr, root_vasp);
    }

    // Rotate the compliance public key for `root_vasp_addr` to `new_public_key`.
    // TODO: allow a VASP to rotate its own compliance key
    public fun rotate_compliance_public_key(root_vasp_addr: address, new_public_key: vector<u8>) {
        Transaction::assert(Vector::length(&new_public_key) == 32, 7004);
        let root_vasp = AccountType::account_metadata<RootVASP>(root_vasp_addr);
        root_vasp.compliance_public_key = new_public_key;
        AccountType::update<RootVASP>(root_vasp_addr, root_vasp);
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be root-vasp called functions
    ///////////////////////////////////////////////////////////////////////////

    public fun create_root_vasp_credential(
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>
    ): RootVASP {
        // NOTE: Only callable in testnet
        Transaction::assert(Testnet::is_testnet(), 10041);
        RootVASP {
           // For testnet, so it should never expire. So set to u64::MAX
           expiration_date: 18446744073709551615,
           human_name,
           base_url,
           compliance_public_key,
        }
    }

    // An account that wishes to become a root VASP account publishes an
    // `AccountType<RootVASP>` resource under their account. However, they
    // are not a root VASP until the association certifies this account
    // type resource.
    public fun apply_for_vasp_root_credential(
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>
    ) {
        // Sanity check for key validity
        Transaction::assert(Vector::length(&compliance_public_key) == 32, 7004);
        AccountType::apply_for<RootVASP>(RootVASP {
            expiration_date: LibraTimestamp::now_microseconds() + cert_lifetime(),
            human_name,
            base_url,
            compliance_public_key,
        }, singleton_addr());
        AccountType::apply_for_granting_capability<ChildVASP>();
    }

    // Child accounts for VASPs are not allowed by default. Calling this
    // function from the root VASP account allows child accounts to be
    // created for the calling VASP account.
    public fun allow_child_accounts() {
        let sender = Transaction::sender();
        AccountType::apply_for_transition_capability<ChildVASP>(sender);
        AccountType::grant_transition_capability<ChildVASP>(sender);
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be parent called functions, and root-vasp-called decertification
    ///////////////////////////////////////////////////////////////////////////

    // Publishes a (to-be-certified) transition under the sending address,
    // with the root authority being at `root_vasp_addr`.
    public fun apply_for_parent_capability() {
        let sender = Transaction::sender();
        Transaction::assert(is_child_vasp(sender), 7002);
        let root_vasp_addr = root_vasp_address(sender);
        // Apply for the ability to transition Empty::T => VASP::ChildVASP
        // accounts with the root authority address being at `root_vasp_addr`
        AccountType::apply_for_transition_capability<ChildVASP>(root_vasp_addr);
    }

    // Certifies the transition capability at `for_address`. The caller
    // must be the root VASP account.
    public fun grant_parent_capability(for_address: address) {
        AccountType::grant_transition_capability<ChildVASP>(for_address);
    }

    // Removes the ability for the account at `addr` to create additional
    // accounts on behalf of the root VASP (which must be the sender). Must
    // be called by the root VASP account.
    public fun remove_parent_capability(addr: address) {
        AccountType::remove_transition_capability<ChildVASP>(addr);
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be-child called functions
    ///////////////////////////////////////////////////////////////////////////

    // Certifies the child VASP account at `addr`. Must be called by a
    // parent or root VASP account. NB that we don't need to worry about
    // account limit restrictions since only an Empty::T account type can
    // be transitioned--therefore its balance must be 0 in order for
    // certification to succeed.
    public fun grant_child_account(addr: address) {
        let root_account_addr = AccountType::transition_cap_root_addr<ChildVASP>(Transaction::sender());
        Transaction::assert(is_root_vasp(root_account_addr), 7001);
        // Transition the child account: Empty::T => VASP::ChildVASP.
        // The ChildVASP account type must be published under the child, but not yet certified
        AccountType::transition<ChildVASP>(addr);
        let child_vasp = AccountType::account_metadata<ChildVASP>(addr);
        // Now mark this account as certified
        child_vasp.is_certified = true;
        AccountType::update<ChildVASP>(addr, child_vasp);
    }

    // Decertify the child VASP account at `addr`. Caller must be the root
    // VASP account.
    public fun decertify_child_account(addr: address) {
        Transaction::assert(!is_parent_vasp(addr), 7003);
        let root_account_addr = AccountType::transition_cap_root_addr<ChildVASP>(Transaction::sender());
        Transaction::assert(is_root_vasp(root_account_addr), 7001);
        let child_vasp = AccountType::account_metadata<ChildVASP>(addr);
        child_vasp.is_certified = false;
        AccountType::update<ChildVASP>(addr, child_vasp);
    }

    // Recertifies a child account that has been previously uncertified.
    public fun recertify_child_account(addr: address) {
        let root_account_addr = AccountType::transition_cap_root_addr<ChildVASP>(Transaction::sender());
        Transaction::assert(is_root_vasp(root_account_addr), 7001);
        // Child cert must be published under the child, but not yet certified
        let child_vasp = AccountType::account_metadata<ChildVASP>(addr);
        child_vasp.is_certified = true;
        AccountType::update<ChildVASP>(addr, child_vasp);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs
    ///////////////////////////////////////////////////////////////////////////

    // Publish the child VASP resource under the sending account. It can
    // only be certified by the root account or a parent account under the
    // same root VASP.
    public fun apply_for_child_vasp_credential(root_vasp_addr: address) {
        let sender = Transaction::sender();
        Transaction::assert(!is_vasp(sender), 7002);
        Transaction::assert(is_root_vasp(root_vasp_addr), 7001);
        AccountType::assert_has_transition_cap<ChildVASP>(root_vasp_addr);
        AccountType::apply_for<ChildVASP>(ChildVASP{ is_certified: false }, root_vasp_addr);
    }

    // The account at address `addr` is a root VASP if the account at that
    // address has a certified AccountType<RootVASP> resource that has not expired.
    public fun is_root_vasp(addr: address): bool {
        if (AccountType::is_a<RootVASP>(addr)) {
            let root_vasp = AccountType::account_metadata<RootVASP>(addr);
            !root_credential_expired(&root_vasp)
        } else {
            false
        }
    }

    // The account at address `addr` is a child VASP if it:
    // 1. Is a (certified) ChildVASP account type; and
    // 2. Its root_address field points to a valid (in the sense of
    // `is_root_vasp`) root VASP account.
    public fun is_child_vasp(addr: address): bool {
        if (AccountType::is_a<ChildVASP>(addr)) {
            is_root_vasp(AccountType::root_address<ChildVASP>(addr)) &&
            AccountType::account_metadata<ChildVASP>(addr).is_certified
        } else false
    }

    // Return the root address for a child VASP account.
    public fun root_vasp_address(addr: address): address {
        if (is_child_vasp(addr)) {
            AccountType::root_address<ChildVASP>(addr)
        } else if (is_root_vasp(addr)) {
            addr
        } else {
            abort 2
        }
    }

    // Return whether `root_vasp_addr` and `child_address` have a valid
    // root-child VASP relationship.
    public fun is_root_child_vasp(root_vasp_addr: address, child_address: address): bool {
        if (is_root_vasp(root_vasp_addr) && is_child_vasp(child_address)) {
            AccountType::root_address<ChildVASP>(child_address) == root_vasp_addr
        } else {
            false
        }
    }

    // Return if the transaction sender is a parent VASP account.
    public fun is_parent_vasp(addr: address): bool {
        AccountType::has_transition_cap<ChildVASP>(addr)
    }

    // Return whether the account at `addr` is a VASP account. An account
    // at address `addr` is a VASP account iff it is a child or root VASP
    // account.
    public fun is_vasp(addr: address): bool {
        is_root_vasp(addr) || is_child_vasp(addr)
    }

    // Return whether or not the root VASP at `root_vasp_addr` allows child
    // accounts.
    public fun allows_child_accounts(root_vasp_addr: address): bool {
        is_root_vasp(root_vasp_addr) &&
        AccountType::has_transition_cap<ChildVASP>(root_vasp_addr)
    }

    ///////////////////////////////////////////////////////////////////////////
    // Root VASP credential field accessors.
    ///////////////////////////////////////////////////////////////////////////
    //.........................................................................
    // These work on all valid (child, parent, root) VASP accounts.
    //.........................................................................

    // Return the human-readable name for the VASP account at `addr`.
    public fun human_name(addr: address): vector<u8> {
        let root_vasp_addr = root_vasp_address(addr);
        let root_vasp = AccountType::account_metadata<RootVASP>(root_vasp_addr);
        *&root_vasp.human_name
    }

    // Return the base URL for the VASP account at `addr`.
    public fun base_url(addr: address): vector<u8> {
        let root_vasp_addr = root_vasp_address(addr);
        let root_vasp = AccountType::account_metadata<RootVASP>(root_vasp_addr);
        *&root_vasp.base_url
    }

    // Return the compliance public key for the VASP account at `addr`.
    public fun compliance_public_key(addr: address): vector<u8> {
        let root_vasp_addr = root_vasp_address(addr);
        let root_vasp = AccountType::account_metadata<RootVASP>(root_vasp_addr);
        *&root_vasp.compliance_public_key
    }

    // Return the expiration date for the VASP account at `addr`.
    public fun expiration_date(addr: address): u64 {
        let root_vasp_addr = root_vasp_address(addr);
        let root_vasp = AccountType::account_metadata<RootVASP>(root_vasp_addr);
        root_vasp.expiration_date
    }

    fun root_credential_expired(root_credential: &RootVASP): bool {
        root_credential.expiration_date < LibraTimestamp::now_microseconds()
    }

    fun assert_sender_is_assoc_vasp_privileged() {
        Transaction::assert(Association::has_privilege<CreationPrivilege>(Transaction::sender()), 7000);
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

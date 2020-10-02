address 0x1 {

/// Module providing well-known addresses and related logic.
///
/// > Note: this module currently defines zero-argument functions like `Self::LIBRA_ROOT_ADDRESS()` using capitalization
/// > in the name, as usually reserved for constants. Eventually, those functions are planned to become actually
/// > global constants, once the Move language supports this feature.
module CoreAddresses {
    use 0x1::Errors;
    use 0x1::Signer;

    /// The address of the libra root account. This account is
    /// created in genesis, and cannot be changed. This address has
    /// ultimate authority over the permissions granted (or removed) from
    /// accounts on-chain.
    public fun LIBRA_ROOT_ADDRESS(): address {
        0xA550C18
    }

    /// The (singleton) address under which the `0x1::Libra::CurrencyInfo` resource for
    /// every registered currency is published. This is the same as the
    /// `LIBRA_ROOT_ADDRESS` but there is no requirement that it must
    /// be this from an operational viewpoint, so this is why this is separated out.
    public fun CURRENCY_INFO_ADDRESS(): address {
        0xA550C18
    }

    /// The account address of the treasury and compliance account in
    /// charge of minting/burning and other day-to-day but privileged
    /// operations. The account at this address is created in genesis.
    public fun TREASURY_COMPLIANCE_ADDRESS(): address {
        0xB1E55ED
    }

    /// The reserved address for transactions inserted by the VM into blocks (e.g.
    /// block metadata transactions). Because the transaction is sent from
    /// the VM, an account _cannot_ exist at the `0x0` address since there
    /// is no signer for the transaction.
    public fun VM_RESERVED_ADDRESS(): address {
        0x0
    }

    /// The operation can only be performed by the account at 0xA550C18 (Libra Root)
    const ELIBRA_ROOT: u64 = 0;
    /// The operation can only be performed by the account at 0xB1E55ED (Treasury & Compliance)
    const ETREASURY_COMPLIANCE: u64 = 1;
    /// The operation can only be performed by the VM
    const EVM: u64 = 2;
    /// The operation can only be performed by the account where currencies are registered
    const ECURRENCY_INFO: u64 = 4; // TODO: This error code and below should decrease by 1.

    /// Assert that the account is the libra root address.
    public fun assert_libra_root(account: &signer) {
        assert(Signer::address_of(account) == LIBRA_ROOT_ADDRESS(), Errors::requires_address(ELIBRA_ROOT))
    }
    spec fun assert_libra_root {
        pragma opaque;
        include AbortsIfNotLibraRoot;
    }

    /// Specifies that a function aborts if the account has not the Libra root address.
    spec schema AbortsIfNotLibraRoot {
        account: signer;
        aborts_if Signer::spec_address_of(account) != LIBRA_ROOT_ADDRESS()
            with Errors::REQUIRES_ADDRESS;
    }

    /// Assert that the signer has the treasury compliance address.
    public fun assert_treasury_compliance(account: &signer) {
        assert(
            Signer::address_of(account) == TREASURY_COMPLIANCE_ADDRESS(),
            Errors::requires_address(ETREASURY_COMPLIANCE)
        )
    }
    spec fun assert_treasury_compliance {
        pragma opaque;
        include AbortsIfNotTreasuryCompliance;
    }

    /// Specifies that a function aborts if the account has not the treasury compliance address.
    spec schema AbortsIfNotTreasuryCompliance {
        account: signer;
        aborts_if Signer::spec_address_of(account) != TREASURY_COMPLIANCE_ADDRESS()
            with Errors::REQUIRES_ADDRESS;
    }

    /// Assert that the signer has the VM reserved address.
    public fun assert_vm(account: &signer) {
        assert(Signer::address_of(account) == VM_RESERVED_ADDRESS(), Errors::requires_address(EVM))
    }
    spec fun assert_vm {
        pragma opaque;
        include AbortsIfNotVM;
    }

    /// Specifies that a function aborts if the account has not the VM reserved address.
    spec schema AbortsIfNotVM {
        account: signer;
        aborts_if Signer::spec_address_of(account) != VM_RESERVED_ADDRESS()
            with Errors::REQUIRES_ADDRESS;
    }

    /// Assert that the signer has the currency info address.
    public fun assert_currency_info(account: &signer) {
        assert(Signer::address_of(account) == CURRENCY_INFO_ADDRESS(), Errors::requires_address(ECURRENCY_INFO))
    }
    spec fun assert_currency_info {
        pragma opaque;
        include AbortsIfNotCurrencyInfo;
    }

    /// Specifies that a function aborts if the account has not the currency info address.
    spec schema AbortsIfNotCurrencyInfo {
        account: signer;
        aborts_if Signer::spec_address_of(account) != CURRENCY_INFO_ADDRESS()
            with Errors::REQUIRES_ADDRESS;
    }

}
}

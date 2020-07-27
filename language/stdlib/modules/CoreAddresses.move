address 0x1 {
module CoreAddresses {
    use 0x1::CoreErrors;
    use 0x1::Signer;

    /// The address of the libra root account. This account is
    /// created in genesis, and cannot be changed. This address has
    /// ultimate authority over the permissions granted (or removed) from
    /// accounts on-chain.
    public fun LIBRA_ROOT_ADDRESS(): address {
        0xA550C18
    }

    /// Specification version of `Self::LIBRA_ROOT_ADDRESS`.
    spec module {
        // DEPRECATED: directly use Move function
        define SPEC_LIBRA_ROOT_ADDRESS(): address {
            0xA550C18
        }
    }

    /// The (singleton) address under which the `0x1::Libra::CurrencyInfo` resource for
    /// every registered currency is published. This is the same as the
    /// `LIBRA_ROOT_ADDRESS` but there is no requirement that it must
    /// be this from an operational viewpoint, so this is why this is separated out.
    public fun CURRENCY_INFO_ADDRESS(): address {
        0xA550C18
    }

    /// Specification version of `Self::CURRENCY_INFO_ADDRESS`.
    spec module {
        // DEPRECATED: directly use Move function
        define SPEC_CURRENCY_INFO_ADDRESS(): address {
            0xA550C18
        }
    }

    /// The account address of the treasury and compliance account in
    /// charge of minting/burning and other day-to-day but privileged
    /// operations. The account at this address is created in genesis.
    public fun TREASURY_COMPLIANCE_ADDRESS(): address {
        0xB1E55ED
    }

    /// Specification version of `Self::TREASURY_COMPLIANCE_ADDRESS`.
    spec module {
        // DEPRECATED: directly use Move function
        define SPEC_TREASURY_COMPLIANCE_ADDRESS(): address {
            0xB1E55ED
        }
    }

    /// The reserved address for transactions inserted by the VM into blocks (e.g.
    /// block metadata transactions). Because the transaction is sent from
    /// the VM, an account _cannot_ exist at the `0x0` address since there
    /// is no signer for the transaction.
    public fun VM_RESERVED_ADDRESS(): address {
        0x0
    }

    /// Specification version of `Self::VM_RESERVED_ADDRESS`.
    spec module {
        // DEPRECATED: directly use Move function
        define SPEC_VM_RESERVED_ADDRESS(): address {
            0x0
        }
    }

    /// # Specification Schemas for Aborts Related to Core Addresses

    /// Note that currently, we do not define these functions on Move level, because it
    /// is better for debugging to have the matching assert directly in the code than indirect via a function.

    /// TODO(wrwg): revisit whether we really cannot write helpers which handle the aborts in Move. Perhaps we
    ///   can have stack traces from the VM which would make this restriction unnecessary.

    /// Specifies that a function aborts if the account is not the Libra root.
    spec schema AbortsIfNotLibraRoot {
        account: signer;
        aborts_if Signer::spec_address_of(account) != LIBRA_ROOT_ADDRESS()
            with CoreErrors::NOT_LIBRA_ROOT_ADDRESS();
    }

    /// Specifies that a function aborts if the account is not the VM reserved address.
    spec schema AbortsIfNotVM {
        account: signer;
        aborts_if Signer::spec_address_of(account) != VM_RESERVED_ADDRESS()
            with CoreErrors::NOT_VM_RESERVED_ADDRESS();
    }

}
}

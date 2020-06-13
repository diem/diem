address 0x1 {
module CoreAddresses {
    /// The address of the root association account. This account is
    /// created in genesis, and cannot be changed. This address has
    /// ultimate authority over the permissions granted (or removed) from
    /// accounts on-chain.
    public fun ASSOCIATION_ROOT_ADDRESS(): address {
        0xA550C18
    }

    /// Specification version of `Self::ASSOCIATION_ROOT_ADDRESS`.
    spec module {
        define SPEC_ASSOCIATION_ROOT_ADDRESS(): address {
            0xA550C18
        }
    }

    /// The (singleton) address under which the `0x1::Libra::CurrencyInfo` resource for
    /// every registered currency is published. This is the same as the
    /// `ASSOCIATION_ROOT_ADDRESS` but there is no requirement that it must
    /// be this from an operational viewpoint, so this is why this is separated out.
    public fun CURRENCY_INFO_ADDRESS(): address {
        0xA550C18
    }

    /// Specification version of `Self::CURRENCY_INFO_ADDRESS`.
    spec module {
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
        define SPEC_VM_RESERVED_ADDRESS(): address {
            0x0
        }
    }
}
}

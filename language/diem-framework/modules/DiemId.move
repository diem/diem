address 0x1 {

/// Module managing Diem ID.
module DiemId {

    use 0x1::Event::{Self, EventHandle};

    /// This resource holds an entity's domain names needed to send and receive payments using diem IDs.
    struct DiemIdDomains has key {
        /// The list of domain names owned by this parent vasp account
        domains: vector<DiemIdDomain>,
    }

    /// Struct to store the limit on-chain
    struct DiemIdDomain has key {
        domain: vector<u8>, // UTF-8 encoded
    }

    struct DiemIdDomainManager has key {
        /// Event handle for `domains` added or removed events. Emitted every time a domain is added
        /// or removed to `domains`
        diem_id_domain_events: EventHandle<Self::DiemIdDomainEvent>,
    }

    struct DiemIdDomainEvent has drop, store {
        /// Whether a domain was added or removed
        removed: bool,
        /// Diem ID Domain string of the account
        domain: DiemIdDomain,
        /// On-chain account address
        address: address,
    }

    const MAX_U64: u128 = 18446744073709551615;

    // Error codes
    /// A credential is not or already published.
    const EDIEMIDDOMAIN: u64 = 0;
    /// A limit is not or already published.
    const ELIMIT: u64 = 1;
    /// Cannot parse this as an ed25519 public key
    const EINVALID_PUBLIC_KEY: u64 = 2;
    /// Cannot parse this as an ed25519 signature (e.g., != 64 bytes)
    const EMALFORMED_METADATA_SIGNATURE: u64 = 3;
    /// Signature does not match message and public key
    const EINVALID_METADATA_SIGNATURE: u64 = 4;
    /// The recipient of a dual attestation payment needs to set a compliance public key
    const EPAYEE_COMPLIANCE_KEY_NOT_SET: u64 = 5;
    /// The recipient of a dual attestation payment needs to set a base URL
    const EPAYEE_BASE_URL_NOT_SET: u64 = 6;

    /// Value of the dual attestation limit at genesis
    const INITIAL_DUAL_ATTESTATION_LIMIT: u64 = 1000;
    /// Suffix of every signed dual attestation message
    const DOMAIN_SEPARATOR: vector<u8> = b"@@$$DIEM_ATTEST$$@@";
    /// A year in microseconds
    const ONE_YEAR: u64 = 31540000000000;
    const U64_MAX: u64 = 18446744073709551615;

    /// Publish a `DiemIdDomains` resource under `created` with an empty `domains`.
    /// Before sending or receiving any payments using Diem IDs, the Treasury Compliance account must send
    /// a transaction that invokes `add_domain_id` to set the `domains` field with a valid domain
    public fun publish_diem_id_domain(
        created: &signer,
        creator: &signer,
    ) {
        Roles::assert_parent_vasp_or_designated_dealer(created);
        Roles::assert_treasury_compliance(creator);
        assert(
            !exists<DiemIdDomains>(Signer::address_of(created)),
            Errors::already_published(EDIEMIDDOMAIN)
        );
        move_to(created, DiemIdDomains {
            domains: Vector::empty(),
        })
    }
    spec fun publish_diem_id_domain {
        include Roles::AbortsIfNotParentVaspOrDesignatedDealer{account: created};
        include Roles::AbortsIfNotTreasuryCompliance{account: creator};
        aborts_if spec_has_diem_id_domains(Signer::spec_address_of(created)) with Errors::ALREADY_PUBLISHED;
    }
    spec define spec_has_diem_id_domains(addr: address): bool {
        exists<DiemIdDomains>(addr)
    }

}
}

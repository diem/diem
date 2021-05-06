address 0x1 {

/// Module managing Diem ID.
module DiemId {

    use 0x1::Event::{Self, EventHandle};
    use 0x1::Vector;
    use 0x1::CoreAddresses;
    use 0x1::Roles;
    use 0x1::Errors;
    use 0x1::Signer;

    /// This resource holds an entity's domain names needed to send and receive payments using diem IDs.
    struct DiemIdDomains has key {
        /// The list of domain names owned by this parent vasp account
        domains: vector<DiemIdDomain>,
    }

    /// Struct to store the limit on-chain
    struct DiemIdDomain has drop, store, copy {
        domain: vector<u8>, // UTF-8 encoded and 63 characters
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
    /// DiemIdDomains resource is not or already published.
    const EDIEMIDDOMAIN: u64 = 0;
    /// DiemIdDomainManager resource is not or already published.
    const EDIEMIDDOMAINMANAGER: u64 = 1;

    /// Publish a `DiemIdDomains` resource under `created` with an empty `domains`.
    /// Before sending or receiving any payments using Diem IDs, the Treasury Compliance account must send
    /// a transaction that invokes `add_domain_id` to set the `domains` field with a valid domain
    public fun publish_diem_id_domains(
        created: &signer,
    ) {
        Roles::assert_parent_vasp_role(created);
        assert(
            !exists<DiemIdDomains>(Signer::address_of(created)),
            Errors::already_published(EDIEMIDDOMAIN)
        );
        move_to(created, DiemIdDomains {
            domains: Vector::empty(),
        })
    }
    spec fun publish_diem_id_domains {
        include Roles::AbortsIfNotParentVasp{account: created};
        aborts_if has_diem_id_domains(Signer::spec_address_of(created)) with Errors::ALREADY_PUBLISHED;
    }

    public fun has_diem_id_domains(addr: address): bool {
        exists<DiemIdDomains>(addr)
    }

    /// Publish a `DiemIdDomainManager` resource under `created` with an empty `diem_id_domain_events`.
    /// When Treasury Compliance account sends a transaction that invokes `update_diem_id_domain`,
    /// a `DiemIdDomainEvent` is emitted and added to `diem_id_domain_events`.
    public fun publish_diem_id_domain_manager(
        created: &signer,
    ) {
        Roles::assert_treasury_compliance(created);
        assert(
            !exists<DiemIdDomainManager>(Signer::address_of(created)),
            Errors::already_published(EDIEMIDDOMAINMANAGER)
        );
        move_to(
            created,
            DiemIdDomainManager {
                diem_id_domain_events: Event::new_event_handle<DiemIdDomainEvent>(created),
            }
        );
    }

    spec define spec_has_diem_id_domain_manager(addr: address): bool {
        exists<DiemIdDomainManager>(addr)
    }

    spec fun publish_diem_id_domain_manager {
        include Roles::AbortsIfNotTreasuryCompliance{account: created};
        aborts_if spec_has_diem_id_domain_manager(Signer::spec_address_of(created)) with Errors::ALREADY_PUBLISHED;
    }

    public fun update_diem_id_domain(
        tc_account: &signer,
        to_update_address: address,
        domain: vector<u8>,
        is_remove: bool
    ) acquires DiemIdDomainManager, DiemIdDomains {
        Roles::assert_treasury_compliance(tc_account);
        let account_domains = borrow_global_mut<DiemIdDomains>(to_update_address);
        let diem_id_domain = DiemIdDomain {
            domain: domain
        };
        Vector::push_back(&mut account_domains.domains, copy diem_id_domain);

        Event::emit_event(
            &mut borrow_global_mut<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()).diem_id_domain_events,
            DiemIdDomainEvent {
                removed: is_remove,
                domain: diem_id_domain,
                address: to_update_address,
            },
        );
    }

    spec fun update_diem_id_domain {
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
    }

}
}

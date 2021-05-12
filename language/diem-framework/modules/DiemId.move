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
        diem_id_domain_events: EventHandle<DiemIdDomainEvent>,
    }

    struct DiemIdDomainEvent has drop, store {
        /// Whether a domain was added or removed
        removed: bool,
        /// Diem ID Domain string of the account
        domain: DiemIdDomain,
        /// On-chain account address
        address: address,
    }

    const DOMAIN_LENGTH: u64 = 63;

    // Error codes
    /// DiemIdDomains resource is not or already published.
    const EDIEM_ID_DOMAIN: u64 = 0;
    /// DiemIdDomainManager resource is not or already published.
    const EDIEM_ID_DOMAIN_MANAGER: u64 = 1;
    /// DiemID domain was not found
    const EDOMAIN_NOT_FOUND: u64 = 2;
    /// DiemID domain already exists
    const EDOMAIN_ALREADY_EXISTS: u64 = 3;
    /// DiemIdDomains resource was not published for a VASP account
    const EDIEM_ID_DOMAINS_NOT_PUBLISHED: u64 = 4;
    /// Invalid domain for DiemIdDomain
    const EINVALID_DIEM_ID_DOMAIN: u64 = 5;

    spec module {
        pragma verify = false;
    }

    fun create_diem_id_domain(domain: vector<u8>): DiemIdDomain {
        assert(Vector::length(&domain) <= DOMAIN_LENGTH, Errors::invalid_argument(EINVALID_DIEM_ID_DOMAIN));
        DiemIdDomain {domain}
    }

    /// Publish a `DiemIdDomains` resource under `created` with an empty `domains`.
    /// Before sending or receiving any payments using Diem IDs, the Treasury Compliance account must send
    /// a transaction that invokes `add_domain_id` to set the `domains` field with a valid domain
    public fun publish_diem_id_domains(
        vasp_account: &signer,
    ) {
        Roles::assert_parent_vasp_role(vasp_account);
        assert(
            !exists<DiemIdDomains>(Signer::address_of(vasp_account)),
            Errors::already_published(EDIEM_ID_DOMAIN)
        );
        move_to(vasp_account, DiemIdDomains {
            domains: Vector::empty(),
        })
    }

    spec fun publish_diem_id_domains {
        include Roles::AbortsIfNotParentVasp{account: vasp_account};
        aborts_if has_diem_id_domains(Signer::spec_address_of(vasp_account)) with Errors::ALREADY_PUBLISHED;
        aborts_if exists<DiemIdDomains>(Signer::address_of(vasp_account)) with Errors::ALREADY_PUBLISHED;
    }

    public fun has_diem_id_domains(addr: address): bool {
        exists<DiemIdDomains>(addr)
    }

    /// Publish a `DiemIdDomainManager` resource under `tc_account` with an empty `diem_id_domain_events`.
    /// When Treasury Compliance account sends a transaction that invokes `update_diem_id_domains`,
    /// a `DiemIdDomainEvent` is emitted and added to `diem_id_domain_events`.
    public fun publish_diem_id_domain_manager(
        tc_account : &signer,
    ) {
        Roles::assert_treasury_compliance(tc_account);
        assert(
            !exists<DiemIdDomainManager>(Signer::address_of(tc_account)),
            Errors::already_published(EDIEM_ID_DOMAIN_MANAGER)
        );
        move_to(
            tc_account,
            DiemIdDomainManager {
                diem_id_domain_events: Event::new_event_handle<DiemIdDomainEvent>(tc_account),
            }
        );
    }

    spec fun publish_diem_id_domain_manager {
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        aborts_if tc_domain_manager_exists() with Errors::ALREADY_PUBLISHED;
    }

    /// When updating DiemIdDomains, a simple duplicate domain check is done.
    /// However, since domains are case insensitive, it is possible by error that two same domains in
    /// different lowercase and uppercase format gets added.
    public fun update_diem_id_domains(
        tc_account: &signer,
        to_update_address: address,
        domain: vector<u8>,
        is_remove: bool
    ) acquires DiemIdDomainManager, DiemIdDomains {
        Roles::assert_treasury_compliance(tc_account);
        if (!exists<DiemIdDomains>(to_update_address)) {
            abort(Errors::not_published(EDIEM_ID_DOMAINS_NOT_PUBLISHED))
        };
        let account_domains = borrow_global_mut<DiemIdDomains>(to_update_address);
        let diem_id_domain = create_diem_id_domain(domain);
        if (is_remove) {
            let (has, index) = Vector::index_of(&account_domains.domains, &diem_id_domain);
            if (has) {
                Vector::remove(&mut account_domains.domains, index);
            } else {
                abort(Errors::invalid_argument(EDOMAIN_NOT_FOUND))
            };
        } else {
            if (!Vector::contains(&account_domains.domains, &diem_id_domain)) {
                Vector::push_back(&mut account_domains.domains, copy diem_id_domain);
            } else {
                abort(Errors::invalid_argument(EDOMAIN_ALREADY_EXISTS))
            }
        };

        Event::emit_event(
            &mut borrow_global_mut<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()).diem_id_domain_events,
            DiemIdDomainEvent {
                removed: is_remove,
                domain: diem_id_domain,
                address: to_update_address,
            },
        );
    }

    spec fun update_diem_id_domains {
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
    }

    public fun has_diem_id_domain(addr: address, domain: vector<u8>): bool acquires DiemIdDomains {
        if (!exists<DiemIdDomains>(addr)) {
            abort(Errors::not_published(EDIEM_ID_DOMAINS_NOT_PUBLISHED))
        };
        let account_domains = borrow_global<DiemIdDomains>(addr);
        let diem_id_domain = create_diem_id_domain(domain);
        Vector::contains(&account_domains.domains, &diem_id_domain)
    }

    public fun tc_domain_manager_exists(): bool {
        exists<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS())
    }

}
}

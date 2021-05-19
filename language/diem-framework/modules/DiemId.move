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
    spec DiemIdDomains {
        /// All `DiemIdDomain`s stored in the `DiemIdDomains` resource are no more than 63 characters long.
        invariant forall i in 0..len(domains): len(domains[i].domain) <= DOMAIN_LENGTH;
        /// The list of `DiemIdDomain`s are a set
        invariant forall i in 0..len(domains):
            forall j in i + 1..len(domains): domains[i] != domains[j];
    }

    /// Struct to store the limit on-chain
    struct DiemIdDomain has drop, store, copy {
        domain: vector<u8>, // UTF-8 encoded and 63 characters
    }
    spec DiemIdDomain {
        /// All `DiemIdDomain`s must be no more than 63 characters long.
        invariant len(domain) <= DOMAIN_LENGTH;
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

    fun create_diem_id_domain(domain: vector<u8>): DiemIdDomain {
        assert(Vector::length(&domain) <= DOMAIN_LENGTH, Errors::invalid_argument(EINVALID_DIEM_ID_DOMAIN));
        DiemIdDomain{ domain }
    }
    spec create_diem_id_domain {
        include CreateDiemIdDomainAbortsIf;
        ensures result == DiemIdDomain { domain };
    }
    spec schema CreateDiemIdDomainAbortsIf {
        domain: vector<u8>;
        aborts_if Vector::length(domain) > DOMAIN_LENGTH with Errors::INVALID_ARGUMENT;
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
    spec publish_diem_id_domains {
        let vasp_addr = Signer::spec_address_of(vasp_account);
        include Roles::AbortsIfNotParentVasp{account: vasp_account};
        include PublishDiemIdDomainsAbortsIf;
        include PublishDiemIdDomainsEnsures;
    }
    spec schema PublishDiemIdDomainsAbortsIf {
        vasp_addr: address;
        aborts_if has_diem_id_domains(vasp_addr) with Errors::ALREADY_PUBLISHED;
    }
    spec schema PublishDiemIdDomainsEnsures {
        vasp_addr: address;
        ensures exists<DiemIdDomains>(vasp_addr);
        ensures Vector::is_empty(global<DiemIdDomains>(vasp_addr).domains);
    }

    public fun has_diem_id_domains(addr: address): bool {
        exists<DiemIdDomains>(addr)
    }
    spec has_diem_id_domains {
        aborts_if false;
        ensures result == exists<DiemIdDomains>(addr);
    }

    /// Publish a `DiemIdDomainManager` resource under `tc_account` with an empty `diem_id_domain_events`.
    /// When Treasury Compliance account sends a transaction that invokes either `add_diem_id_domain` or
    /// `remove_diem_id_domain`, a `DiemIdDomainEvent` is emitted and added to `diem_id_domain_events`.
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
    spec publish_diem_id_domain_manager {
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        aborts_if tc_domain_manager_exists() with Errors::ALREADY_PUBLISHED;
        ensures exists<DiemIdDomainManager>(Signer::spec_address_of(tc_account));
        modifies global<DiemIdDomainManager>(Signer::spec_address_of(tc_account));
    }

    /// Add a DiemIdDomain to a parent VASP's DiemIdDomains resource.
    /// When updating DiemIdDomains, a simple duplicate domain check is done.
    /// However, since domains are case insensitive, it is possible by error that two same domains in
    /// different lowercase and uppercase format gets added.
    public fun add_diem_id_domain(
        tc_account: &signer,
        address: address,
        domain: vector<u8>,
    ) acquires DiemIdDomainManager, DiemIdDomains {
        Roles::assert_treasury_compliance(tc_account);
        assert(tc_domain_manager_exists(), Errors::not_published(EDIEM_ID_DOMAIN_MANAGER));
        assert(
            exists<DiemIdDomains>(address),
            Errors::not_published(EDIEM_ID_DOMAINS_NOT_PUBLISHED)
        );

        let account_domains = borrow_global_mut<DiemIdDomains>(address);
        let diem_id_domain = create_diem_id_domain(domain);

        assert(
            !Vector::contains(&account_domains.domains, &diem_id_domain),
            Errors::invalid_argument(EDOMAIN_ALREADY_EXISTS)
        );

        Vector::push_back(&mut account_domains.domains, copy diem_id_domain);

        Event::emit_event(
            &mut borrow_global_mut<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()).diem_id_domain_events,
            DiemIdDomainEvent {
                removed: false,
                domain: diem_id_domain,
                address,
            },
        );
    }
    spec add_diem_id_domain {
        include AddDiemIdDomainAbortsIf;
        include AddDiemIdDomainEnsures;
        include AddDiemIdDomainEmits;
    }
    spec schema AddDiemIdDomainAbortsIf {
        tc_account: signer;
        address: address;
        domain: vector<u8>;
        let domains = global<DiemIdDomains>(address).domains;
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include CreateDiemIdDomainAbortsIf;
        aborts_if !exists<DiemIdDomains>(address) with Errors::NOT_PUBLISHED;
        aborts_if !tc_domain_manager_exists() with Errors::NOT_PUBLISHED;
        aborts_if contains(domains, DiemIdDomain { domain }) with Errors::INVALID_ARGUMENT;
    }
    spec schema AddDiemIdDomainEnsures {
        address: address;
        domain: vector<u8>;
        let post domains = global<DiemIdDomains>(address).domains;
        ensures contains(domains, DiemIdDomain { domain });
    }
    spec schema AddDiemIdDomainEmits {
        address: address;
        domain: vector<u8>;
        let handle = global<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()).diem_id_domain_events;
        let msg = DiemIdDomainEvent {
            removed: false,
            domain: DiemIdDomain { domain },
            address,
        };
        emits msg to handle;
    }

    /// Remove a DiemIdDomain from a parent VASP's DiemIdDomains resource.
    public fun remove_diem_id_domain(
        tc_account: &signer,
        address: address,
        domain: vector<u8>,
    ) acquires DiemIdDomainManager, DiemIdDomains {
        Roles::assert_treasury_compliance(tc_account);
        assert(tc_domain_manager_exists(), Errors::not_published(EDIEM_ID_DOMAIN_MANAGER));
        assert(
            exists<DiemIdDomains>(address),
            Errors::not_published(EDIEM_ID_DOMAINS_NOT_PUBLISHED)
        );

        let account_domains = borrow_global_mut<DiemIdDomains>(address);
        let diem_id_domain = create_diem_id_domain(domain);

        let (has, index) = Vector::index_of(&account_domains.domains, &diem_id_domain);
        if (has) {
            Vector::remove(&mut account_domains.domains, index);
        } else {
            abort Errors::invalid_argument(EDOMAIN_NOT_FOUND)
        };

        Event::emit_event(
            &mut borrow_global_mut<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()).diem_id_domain_events,
            DiemIdDomainEvent {
                removed: true,
                domain: diem_id_domain,
                address: address,
            },
        );
    }
    spec remove_diem_id_domain {
        include RemoveDiemIdDomainAbortsIf;
        include RemoveDiemIdDomainEnsures;
        include RemoveDiemIdDomainEmits;
    }
    spec schema RemoveDiemIdDomainAbortsIf {
        tc_account: signer;
        address: address;
        domain: vector<u8>;
        let domains = global<DiemIdDomains>(address).domains;
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        include CreateDiemIdDomainAbortsIf;
        aborts_if !exists<DiemIdDomains>(address) with Errors::NOT_PUBLISHED;
        aborts_if !tc_domain_manager_exists() with Errors::NOT_PUBLISHED;
        aborts_if !contains(domains, DiemIdDomain { domain }) with Errors::INVALID_ARGUMENT;
    }
    spec schema RemoveDiemIdDomainEnsures {
        address: address;
        domain: vector<u8>;
        let post domains = global<DiemIdDomains>(address).domains;
        ensures !contains(domains, DiemIdDomain { domain });
    }
    spec schema RemoveDiemIdDomainEmits {
        tc_account: signer;
        address: address;
        domain: vector<u8>;
        let handle = global<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS()).diem_id_domain_events;
        let msg = DiemIdDomainEvent {
            removed: true,
            domain: DiemIdDomain { domain },
            address,
        };
        emits msg to handle;
    }

    public fun has_diem_id_domain(addr: address, domain: vector<u8>): bool acquires DiemIdDomains {
        assert(
            exists<DiemIdDomains>(addr),
            Errors::not_published(EDIEM_ID_DOMAINS_NOT_PUBLISHED)
        );
        let account_domains = borrow_global<DiemIdDomains>(addr);
        let diem_id_domain = create_diem_id_domain(domain);
        Vector::contains(&account_domains.domains, &diem_id_domain)
    }
    spec has_diem_id_domain {
        include HasDiemIdDomainAbortsIf;
        let id_domain = DiemIdDomain { domain };
        ensures result == contains(global<DiemIdDomains>(addr).domains, id_domain);
    }
    spec schema HasDiemIdDomainAbortsIf {
        addr: address;
        domain: vector<u8>;
        include CreateDiemIdDomainAbortsIf;
        aborts_if !exists<DiemIdDomains>(addr) with Errors::NOT_PUBLISHED;
    }

    public fun tc_domain_manager_exists(): bool {
        exists<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS())
    }
    spec tc_domain_manager_exists {
        aborts_if false;
        ensures result == exists<DiemIdDomainManager>(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS());
    }
}
}

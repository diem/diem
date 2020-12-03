address 0x1 {

/// Module which manages freezing of accounts.
module AccountFreezing {
    use 0x1::Event::{Self, EventHandle};
    use 0x1::Errors;
    use 0x1::DiemTimestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Roles;

    resource struct FreezingBit {
        /// If `is_frozen` is set true, the account cannot be used to send transactions or receive funds
        is_frozen: bool,
    }

    resource struct FreezeEventsHolder {
        freeze_event_handle: EventHandle<FreezeAccountEvent>,
        unfreeze_event_handle: EventHandle<UnfreezeAccountEvent>,
    }

    /// Message for freeze account events
    struct FreezeAccountEvent {
        /// The address that initiated freeze txn
        initiator_address: address,
        /// The address that was frozen
        frozen_address: address,
    }

    /// Message for unfreeze account events
    struct UnfreezeAccountEvent {
        /// The address that initiated unfreeze txn
        initiator_address: address,
        /// The address that was unfrozen
        unfrozen_address: address,
    }

    /// A property expected of the `FreezeEventsHolder` resource didn't hold
    const EFREEZE_EVENTS_HOLDER: u64 = 1;
    /// The `FreezingBit` resource is in an invalid state
    const EFREEZING_BIT: u64 = 2;
    /// An attempt to freeze the Diem Root account was attempted
    const ECANNOT_FREEZE_DIEM_ROOT: u64 = 3;
    /// An attempt to freeze the Treasury & Compliance account was attempted
    const ECANNOT_FREEZE_TC: u64 = 4;
    /// The account is frozen
    const EACCOUNT_FROZEN: u64 = 5;

    public fun initialize(dr_account: &signer) {
        DiemTimestamp::assert_genesis();
        CoreAddresses::assert_diem_root(dr_account);
        assert(
            !exists<FreezeEventsHolder>(Signer::address_of(dr_account)),
            Errors::already_published(EFREEZE_EVENTS_HOLDER)
        );
        move_to(dr_account, FreezeEventsHolder {
            freeze_event_handle: Event::new_event_handle(dr_account),
            unfreeze_event_handle: Event::new_event_handle(dr_account),
        });
    }
    spec fun initialize {
        include DiemTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotDiemRoot{account: dr_account};
        let addr = Signer::spec_address_of(dr_account);
        aborts_if exists<FreezeEventsHolder>(addr) with Errors::ALREADY_PUBLISHED;
        ensures exists<FreezeEventsHolder>(addr);
    }

    public fun create(account: &signer) {
        let addr = Signer::address_of(account);
        assert(!exists<FreezingBit>(addr), Errors::already_published(EFREEZING_BIT));
        move_to(account, FreezingBit { is_frozen: false })
    }
    spec fun create {
        let addr = Signer::spec_address_of(account);
        modifies global<FreezingBit>(addr);
        aborts_if exists<FreezingBit>(addr) with Errors::ALREADY_PUBLISHED;
        ensures spec_account_is_not_frozen(addr);
    }

    /// Freeze the account at `addr`.
    public fun freeze_account(
        account: &signer,
        frozen_address: address,
    )
    acquires FreezingBit, FreezeEventsHolder {
        DiemTimestamp::assert_operating();
        Roles::assert_treasury_compliance(account);
        // The diem root account and TC cannot be frozen
        assert(frozen_address != CoreAddresses::DIEM_ROOT_ADDRESS(), Errors::invalid_argument(ECANNOT_FREEZE_DIEM_ROOT));
        assert(frozen_address != CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), Errors::invalid_argument(ECANNOT_FREEZE_TC));
        assert(exists<FreezingBit>(frozen_address), Errors::not_published(EFREEZING_BIT));
        borrow_global_mut<FreezingBit>(frozen_address).is_frozen = true;
        let initiator_address = Signer::address_of(account);
        Event::emit_event<FreezeAccountEvent>(
            &mut borrow_global_mut<FreezeEventsHolder>(CoreAddresses::DIEM_ROOT_ADDRESS()).freeze_event_handle,
            FreezeAccountEvent {
                initiator_address,
                frozen_address
            },
        );
    }
    spec fun freeze_account {
        include DiemTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotTreasuryCompliance;
        aborts_if frozen_address == CoreAddresses::DIEM_ROOT_ADDRESS() with Errors::INVALID_ARGUMENT;
        aborts_if frozen_address == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS() with Errors::INVALID_ARGUMENT;
        aborts_if !exists<FreezingBit>(frozen_address) with Errors::NOT_PUBLISHED;
        ensures spec_account_is_frozen(frozen_address);
    }

    /// Unfreeze the account at `addr`.
    public fun unfreeze_account(
        account: &signer,
        unfrozen_address: address,
    )
    acquires FreezingBit, FreezeEventsHolder {
        DiemTimestamp::assert_operating();
        Roles::assert_treasury_compliance(account);
        assert(exists<FreezingBit>(unfrozen_address), Errors::not_published(EFREEZING_BIT));
        borrow_global_mut<FreezingBit>(unfrozen_address).is_frozen = false;
        let initiator_address = Signer::address_of(account);
        Event::emit_event<UnfreezeAccountEvent>(
            &mut borrow_global_mut<FreezeEventsHolder>(CoreAddresses::DIEM_ROOT_ADDRESS()).unfreeze_event_handle,
            UnfreezeAccountEvent {
                initiator_address,
                unfrozen_address
            },
        );
    }
    spec fun unfreeze_account {
        include DiemTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotTreasuryCompliance;
        aborts_if !exists<FreezingBit>(unfrozen_address) with Errors::NOT_PUBLISHED;
        ensures !spec_account_is_frozen(unfrozen_address);
    }

    /// Returns if the account at `addr` is frozen.
    public fun account_is_frozen(addr: address): bool
    acquires FreezingBit {
        exists<FreezingBit>(addr) && borrow_global<FreezingBit>(addr).is_frozen
     }
    spec fun account_is_frozen {
        aborts_if false;
        pragma opaque = true;
        ensures result == spec_account_is_frozen(addr);
    }

    /// Assert that an account is not frozen.
    public fun assert_not_frozen(account: address) acquires FreezingBit {
        assert(!account_is_frozen(account), Errors::invalid_state(EACCOUNT_FROZEN));
    }
    spec fun assert_not_frozen {
        pragma opaque;
        include AbortsIfFrozen;
    }
    spec schema AbortsIfFrozen {
        account: address;
        aborts_if spec_account_is_frozen(account) with Errors::INVALID_STATE;
    }


    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Initialization
    spec module {
        /// `FreezeEventsHolder` always exists after genesis.
        invariant [global] DiemTimestamp::is_operating() ==>
            exists<FreezeEventsHolder>(CoreAddresses::DIEM_ROOT_ADDRESS());
    }

    /// # Access Control
    spec module {
        /// The account of DiemRoot is not freezable [[F1]][ROLE].
        /// After genesis, FreezingBit of DiemRoot is always false.
        invariant [global] DiemTimestamp::is_operating() ==>
            spec_account_is_not_frozen(CoreAddresses::DIEM_ROOT_ADDRESS());

        /// The account of TreasuryCompliance is not freezable [[F2]][ROLE].
        /// After genesis, FreezingBit of TreasuryCompliance is always false.
        invariant [global] DiemTimestamp::is_operating() ==>
            spec_account_is_not_frozen(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS());

        /// resource struct FreezingBit persists
        invariant update [global] forall addr: address where old(exists<FreezingBit>(addr)): exists<FreezingBit>(addr);

        /// resource struct FreezeEventsHolder is there forever after initialization
        invariant update [global] DiemTimestamp::is_operating() ==> exists<FreezeEventsHolder>(CoreAddresses::DIEM_ROOT_ADDRESS());

        /// The permission "{Freeze,Unfreeze}Account" is granted to TreasuryCompliance only [[H7]][PERMISSION].
        apply Roles::AbortsIfNotTreasuryCompliance to freeze_account, unfreeze_account;

        /// Only (un)freeze functions can change the freezing bits of accounts [[H7]][PERMISSION].
        apply FreezingBitRemainsSame to * except freeze_account, unfreeze_account;
    }

    spec schema FreezingBitRemainsSame {
        ensures forall addr: address where old(exists<FreezingBit>(addr)):
            global<FreezingBit>(addr).is_frozen == old(global<FreezingBit>(addr).is_frozen);
    }

    /// # Helper Functions
    spec module {

        define spec_account_is_frozen(addr: address): bool {
            exists<FreezingBit>(addr) && global<FreezingBit>(addr).is_frozen
        }

        define spec_account_is_not_frozen(addr: address): bool {
            exists<FreezingBit>(addr) && !global<FreezingBit>(addr).is_frozen
        }
    }

}
}

address 0x1 {
module AccountFreezing {
    use 0x1::Event::{Self, EventHandle};
    use 0x1::Errors;
    use 0x1::LibraTimestamp;
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

    const EFREEZE_EVENTS_HOLDER: u64 = 1;
    const EFREEZING_BIT: u64 = 2;
    const ECANNOT_FREEZE_LIBRA_ROOT: u64 = 3;
    const ECANNOT_FREEZE_TC: u64 = 4;
    const ENOT_ABLE_TO_UNFREEZE: u64 = 5;
    const EACCOUNT_FROZEN: u64 = 6;

    public fun initialize(lr_account: &signer) {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        assert(
            !exists<FreezeEventsHolder>(Signer::address_of(lr_account)),
            Errors::already_published(EFREEZE_EVENTS_HOLDER)
        );
        move_to(lr_account, FreezeEventsHolder {
            freeze_event_handle: Event::new_event_handle(lr_account),
            unfreeze_event_handle: Event::new_event_handle(lr_account),
        });
    }
    spec fun initialize {
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        let addr = Signer::spec_address_of(lr_account);
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
        aborts_if exists<FreezingBit>(addr) with Errors::ALREADY_PUBLISHED;
        ensures spec_account_is_not_frozen(addr);
    }

    /// Freeze the account at `addr`.
    public fun freeze_account(
        account: &signer,
        frozen_address: address,
    )
    acquires FreezingBit, FreezeEventsHolder {
        LibraTimestamp::assert_operating();
        Roles::assert_treasury_compliance(account);
        // The libra root account and TC cannot be frozen
        assert(frozen_address != CoreAddresses::LIBRA_ROOT_ADDRESS(), Errors::invalid_argument(ECANNOT_FREEZE_LIBRA_ROOT));
        assert(frozen_address != CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), Errors::invalid_argument(ECANNOT_FREEZE_TC));
        assert(exists<FreezingBit>(frozen_address), Errors::not_published(EFREEZING_BIT));
        borrow_global_mut<FreezingBit>(frozen_address).is_frozen = true;
        let initiator_address = Signer::address_of(account);
        Event::emit_event<FreezeAccountEvent>(
            &mut borrow_global_mut<FreezeEventsHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS()).freeze_event_handle,
            FreezeAccountEvent {
                initiator_address,
                frozen_address
            },
        );
    }
    spec fun freeze_account {
        include LibraTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotTreasuryCompliance;
        aborts_if frozen_address == CoreAddresses::LIBRA_ROOT_ADDRESS() with Errors::INVALID_ARGUMENT;
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
        LibraTimestamp::assert_operating();
        Roles::assert_treasury_compliance(account);
        assert(exists<FreezingBit>(unfrozen_address), Errors::not_published(EFREEZING_BIT));
        borrow_global_mut<FreezingBit>(unfrozen_address).is_frozen = false;
        let initiator_address = Signer::address_of(account);
        Event::emit_event<UnfreezeAccountEvent>(
            &mut borrow_global_mut<FreezeEventsHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS()).unfreeze_event_handle,
            UnfreezeAccountEvent {
                initiator_address,
                unfrozen_address
            },
        );
    }
    spec fun unfreeze_account {
        include LibraTimestamp::AbortsIfNotOperating;
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

    spec module {
        pragma verify = true;

        define spec_account_is_frozen(addr: address): bool {
            exists<FreezingBit>(addr) && global<FreezingBit>(addr).is_frozen
        }

        define spec_account_is_not_frozen(addr: address): bool {
            exists<FreezingBit>(addr) && !global<FreezingBit>(addr).is_frozen
        }

        /// FreezeEventsHolder always exists after genesis.
        invariant [global] LibraTimestamp::is_operating() ==>
            exists<FreezeEventsHolder>(CoreAddresses::LIBRA_ROOT_ADDRESS());

        /// The account of LibraRoot is not freezable [G2].
        /// After genesis, FreezingBit of LibraRoot is always false.
        invariant [global] LibraTimestamp::is_operating() ==>
            spec_account_is_not_frozen(CoreAddresses::LIBRA_ROOT_ADDRESS());

        /// The account of TreasuryCompliance is not freezable [G3].
        /// After genesis, FreezingBit of TreasuryCompliance is always false.
        invariant [global] LibraTimestamp::is_operating() ==>
            spec_account_is_not_frozen(CoreAddresses::TREASURY_COMPLIANCE_ADDRESS());

        /// The permission "{Freeze,Unfreeze}Account" is granted to TreasuryCompliance [B17].
        apply Roles::AbortsIfNotTreasuryCompliance to freeze_account, unfreeze_account;

        // TODO: Need to decide the freezability of the roles such as Validator, ValidatorOperator, DesginatedDealer.
    }
}
}

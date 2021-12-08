#[test_only]
module DiemFramework::AccountLimitsTests {
    use Std::Signer;
    use DiemFramework::Genesis;
    use DiemFramework::AccountLimits;
    use DiemFramework::XUS::XUS;
    use DiemFramework::Roles;

    struct Hold<T> has key { x: T }
    public fun hold<T: store>(account: &signer, x: T) {
        move_to(account, Hold<T>{ x })
    }

    fun setup(dr: &signer, tc: &signer, bob: &signer) {
        Genesis::setup(dr, tc);

        Roles::new_parent_vasp_role(tc, bob);

        AccountLimits::publish_unrestricted_limits_for_testing<XUS>(bob);
        AccountLimits::publish_window<XUS>(dr, bob, Signer::address_of(bob));
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    #[expected_failure(abort_code = 1)]
    fun grant_mutation_capability_after_genesis(dr: signer, tc: signer, bob: signer) {
        Genesis::setup(&dr, &tc);

        hold(&bob, AccountLimits::grant_mutation_capability(&bob));
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    fun publish_window(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    #[expected_failure(abort_code = 262)]
    fun publish_window_twice(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);

        AccountLimits::publish_window<XUS>(&dr, &bob, Signer::address_of(&bob));
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    #[expected_failure(abort_code = 2)]
    fun publish_window_non_diem_root(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        AccountLimits::publish_window<XUS>(&bob, &bob, Signer::address_of(&bob));
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    #[expected_failure(abort_code = 5)]
    fun publish_window_non_existent_limit_address(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        AccountLimits::publish_window<XUS>(&dr, &bob, @0x42 /* non-exsistent */);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    #[expected_failure(abort_code = 6)]
    fun publish_unrestricted_limits_for_testing_twice(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        AccountLimits::publish_unrestricted_limits_for_testing<XUS>(&bob);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    fun update_limits_definition_1(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        AccountLimits::update_limits_definition<XUS>(
            &tc,
            Signer::address_of(&bob),
            100, /* new_max_inflow */
            200, /* new_max_outflow */
            150, /* new_max_holding_balance */
            10000, /* new_time_period */
        )
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    fun update_limits_definition_2(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        AccountLimits::update_limits_definition<XUS>(
            &tc,
            Signer::address_of(&bob),
            0, /* new_max_inflow */
            0, /* new_max_outflow */
            150, /* new_max_holding_balance */
            10000, /* new_time_period */
        )
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    fun update_limits_definition_twice(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        AccountLimits::update_limits_definition<XUS>(
            &tc,
            Signer::address_of(&bob),
            100, /* new_max_inflow */
            200, /* new_max_outflow */
            150, /* new_max_holding_balance */
            10000, /* new_time_period */
        );
        AccountLimits::update_limits_definition<XUS>(
            &tc,
            Signer::address_of(&bob),
            0, /* new_max_inflow */
            0, /* new_max_outflow */
            150, /* new_max_holding_balance */
            10000, /* new_time_period */
        )
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    #[expected_failure(abort_code = 258)]
    fun update_limits_definition_non_tc(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        AccountLimits::update_limits_definition<XUS>(
            &dr,
            Signer::address_of(&bob),
            100, /* new_max_inflow */
            200, /* new_max_outflow */
            150, /* new_max_holding_balance */
            10000, /* new_time_period */
        )
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    #[expected_failure(abort_code = 5)]
    fun update_limits_definition_non_exsistent(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        AccountLimits::update_limits_definition<XUS>(
            &tc,
            @0x42, // non-exsistent
            100, /* new_max_inflow */
            200, /* new_max_outflow */
            150, /* new_max_holding_balance */
            10000, /* new_time_period */
        )
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    fun update_window_info(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        let bob_addr = Signer::address_of(&bob);
        AccountLimits::update_window_info<XUS>(
            &tc,
            bob_addr,
            120,
            bob_addr,
        );
        AccountLimits::update_window_info<XUS>(
            &tc,
            bob_addr,
            0,
            bob_addr,
        );
        AccountLimits::update_window_info<XUS>(
            &tc,
            bob_addr,
            120,
            bob_addr,
        );
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    #[expected_failure(abort_code = 258)]
    fun update_window_info_non_tc(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        let bob_addr = Signer::address_of(&bob);
        AccountLimits::update_window_info<XUS>(
            &dr,
            bob_addr,
            120,
            bob_addr,
        );
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, bob = @0x2)]
    fun has_limits_published(dr: signer, tc: signer, bob: signer) {
        setup(&dr, &tc, &bob);
        assert!(AccountLimits::has_limits_published<XUS>(Signer::address_of(&bob)), 1);
        assert!(!AccountLimits::has_limits_published<XUS>(@0x42 /* non-exsistent */), 3);
    }
}

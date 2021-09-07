#[test_only]
module DiemFramework::ValidatorOperatorConfigTests {
    use DiemFramework::ValidatorOperatorConfig as VOC;
    use DiemFramework::Roles;
    use DiemFramework::Genesis;
    use Std::UnitTest;
    use Std::Vector;
    use Std::Signer;

    fun get_signer(): signer {
        Vector::pop_back(&mut UnitTest::create_signers_for_testing(1))
    }

    #[test]
    #[expected_failure(abort_code = 257)]
    fun publish_pre_genesis() {
        let s = get_signer();
        VOC::publish(&s, &s, x"");
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 2)]
    fun publish_post_genesis_non_dr(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = get_signer();
        VOC::publish(&s, &tc, x"");
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 5)]
    fun publish_post_genesis_non_validator_operator(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = get_signer();
        VOC::publish(&s, &dr, x"");
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    fun publish_post_genesis(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = get_signer();
        Roles::new_validator_operator_role(&dr, &s);
        VOC::publish(&s, &dr, x"");
        assert(VOC::has_validator_operator_config(Signer::address_of(&s)), 0);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 6)]
    fun publish_post_genesis_double_publish(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = get_signer();
        Roles::new_validator_operator_role(&dr, &s);
        VOC::publish(&s, &dr, x"");
        VOC::publish(&s, &dr, x"");
    }

    #[test]
    #[expected_failure(abort_code = 5)]
    fun get_human_name_not_validator_operator() {
        VOC::get_human_name(@0x1);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    fun get_human_name(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = get_signer();
        Roles::new_validator_operator_role(&dr, &s);
        VOC::publish(&s, &dr, b"test");
        assert(VOC::get_human_name(Signer::address_of(&s)) == b"test", 0);
    }
}

#[test_only]
module DiemFramework::ValidatorConfigTests {
    use DiemFramework::ValidatorConfig as VC;
    use DiemFramework::ValidatorOperatorConfig as VOC;
    use DiemFramework::Roles;
    use DiemFramework::Genesis;
    use Std::UnitTest;
    use Std::Vector;
    use Std::Signer;

    const VALID_PUBKEY: vector<u8> = x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";

    fun signer_at(index: u64): signer {
        let signers = UnitTest::create_signers_for_testing(index + 1);
        Vector::pop_back(&mut signers)
    }

    #[test]
    #[expected_failure(abort_code = 257)]
    fun publish_pre_genesis() {
        let s = signer_at(0);
        VC::publish(&s, &s, x"");
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 2)]
    fun publish_post_genesis_non_dr(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = signer_at(0);
        VC::publish(&s, &tc, x"");
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 5)]
    fun publish_post_genesis_non_validator(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = signer_at(0);
        VC::publish(&s, &dr, x"");
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    fun publish_post_genesis(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = signer_at(0);
        Roles::new_validator_role(&dr, &s);
        VC::publish(&s, &dr, x"");
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 6)]
    fun publish_post_genesis_double_publish(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = signer_at(0);
        Roles::new_validator_role(&dr, &s);
        VC::publish(&s, &dr, x"");
        VC::publish(&s, &dr, x"");
    }

    #[test]
    #[expected_failure(abort_code = 5)]
    fun set_operator_not_validator() {
        let s = signer_at(0);
        VC::set_operator(&s, @0x1);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 775)]
    fun set_operator_not_operator(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = signer_at(0);
        Roles::new_validator_role(&dr, &s);
        VC::set_operator(&s, @0x1);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 5)]
    fun set_operator_no_validator_config_has_role(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let validator = signer_at(0);
        let operator = signer_at(1);
        Roles::new_validator_role(&dr, &validator);
        VOC::publish(&operator, &dr, x"");
        VC::set_operator(&validator, Signer::address_of(&operator));
    }

    #[test_only]
    fun set_operator(dr: &signer, validator: &signer, operator: &signer) {
        let operator_addr = Signer::address_of(operator);
        Roles::new_validator_role(dr, validator);
        Roles::new_validator_operator_role(dr, operator);
        VC::publish(validator, dr, x"FF");
        VOC::publish(operator, dr, x"FFFF");
        VC::set_operator(validator, operator_addr);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    fun set_operator_correct(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let validator = signer_at(0);
        let operator = signer_at(1);
        let operator_addr = Signer::address_of(&operator);
        let validator_addr = Signer::address_of(&validator);
        set_operator(&dr, &validator, &operator);
        assert(VC::get_operator(validator_addr) == operator_addr, 0);
    }

    #[test]
    #[expected_failure(abort_code = 5)]
    fun remove_operator_not_validator() {
        let s = signer_at(0);
        VC::remove_operator(&s);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 5)]
    fun remove_operator_no_config(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let s = signer_at(0);
        Roles::new_validator_role(&dr, &s);
        VC::remove_operator(&s);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 7)]
    fun remove_operator_correct(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let validator = signer_at(0);
        let operator = signer_at(1);
        let operator_addr = Signer::address_of(&operator);
        let validator_addr = Signer::address_of(&validator);
        set_operator(&dr, &validator, &operator);
        assert(VC::get_operator(validator_addr) == operator_addr, 0);
        VC::remove_operator(&validator);
        VC::get_operator(validator_addr);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 263)]
    fun set_config_operator_neq_operator(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let validator = signer_at(0);
        let operator = signer_at(1);
        let other_operator = signer_at(2);
        let validator_addr = Signer::address_of(&validator);
        set_operator(&dr, &validator, &operator);
        VC::set_config(&other_operator, validator_addr, x"", x"", x"")
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 519)]
    fun set_config_invalid_consensus_pubkey(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let validator = signer_at(0);
        let operator = signer_at(1);
        let validator_addr = Signer::address_of(&validator);
        set_operator(&dr, &validator, &operator);
        VC::set_config(&operator, validator_addr, x"", x"", x"")
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    fun set_config_correct(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let validator = signer_at(0);
        let operator = signer_at(1);
        let validator_addr = Signer::address_of(&validator);
        let operator_addr = Signer::address_of(&operator);
        set_operator(&dr, &validator, &operator);
        VC::set_config(&operator, validator_addr, VALID_PUBKEY, x"AA", x"BB");
        assert(VC::is_valid(validator_addr), 0);
        assert(VC::get_human_name(validator_addr) == x"FF", 1);
        assert(VC::get_operator(validator_addr) == operator_addr, 2);
        let config = VC::get_config(validator_addr);
        assert(*VC::get_consensus_pubkey(&config) == VALID_PUBKEY, 3);
        assert(*VC::get_validator_network_addresses(&config) == x"AA", 4);
    }

    #[test]
    #[expected_failure(abort_code = 5)]
    fun get_config_not_validator() {
        VC::get_config(@0x1);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 7)]
    fun get_config_not_set(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let validator = signer_at(0);
        let operator = signer_at(1);
        let validator_addr = Signer::address_of(&validator);
        set_operator(&dr, &validator, &operator);
        VC::get_config(validator_addr);
    }

    #[test]
    #[expected_failure(abort_code = 5)]
    fun get_human_name_not_validator() {
        VC::get_human_name(@0x1);
    }

    #[test]
    #[expected_failure(abort_code = 5)]
    fun get_operator_not_validator() {
        VC::get_operator(@0x1);
    }

    #[test(tc = @TreasuryCompliance, dr = @DiemRoot)]
    #[expected_failure(abort_code = 7)]
    fun get_operator_not_set(tc: signer, dr: signer) {
        Genesis::setup(&dr, &tc);
        let validator = signer_at(0);
        let validator_addr = Signer::address_of(&validator);

        Roles::new_validator_role(&dr, &validator);
        VC::publish(&validator, &dr, x"FF");

        VC::get_operator(validator_addr);
    }
}

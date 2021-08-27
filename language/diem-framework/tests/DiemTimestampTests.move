#[test_only]
module DiemFramework::DiemTimestampTests {
    use DiemFramework::Genesis;
    use DiemFramework::DiemTimestamp;
    use Std::Vector;
    use Std::UnitTest;

    fun get_signer(): signer {
        Vector::pop_back(&mut UnitTest::create_signers_for_testing(1))
    }

    #[test]
    #[expected_failure(abort_code = 2)]
    fun set_time_has_started_non_dr_pre_genesis() {
        let s = get_signer();
        DiemTimestamp::set_time_has_started_for_testing(&s);
    }

    #[test(dr = @DiemRoot)]
    fun set_time_has_started_dr_pre_genesis(dr: signer) {
        DiemTimestamp::set_time_has_started_for_testing(&dr);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    #[expected_failure(abort_code = 1)]
    fun set_time_has_started_dr_post_genesis(dr: signer, tc: signer) {
        Genesis::setup(&dr, &tc);
        DiemTimestamp::set_time_has_started_for_testing(&dr);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    #[expected_failure(abort_code = 1)]
    fun set_time_has_started_non_dr_post_genesis(dr: signer, tc: signer) {
        Genesis::setup(&dr, &tc);
        let s = get_signer();
        DiemTimestamp::set_time_has_started_for_testing(&s);
    }

    #[test]
    #[expected_failure(abort_code = 257)]
    fun update_global_time_pre_genesis() {
        let s = get_signer();
        DiemTimestamp::update_global_time(&s, @0x0, 0);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    #[expected_failure(abort_code = 514)]
    fun update_global_time_post_genesis_non_vm(dr: signer, tc: signer) {
        Genesis::setup(&dr, &tc);
        DiemTimestamp::update_global_time(&dr, @0x1, 0);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, vm = @VMReserved)]
    fun update_global_time_post_genesis_vm_nil_proposer_equal_timestamp(dr: signer, tc: signer, vm: signer) {
        Genesis::setup(&dr, &tc);
        assert(DiemTimestamp::now_microseconds() == 0, 0);
        DiemTimestamp::update_global_time(&vm, @VMReserved, 0);
        assert(DiemTimestamp::now_microseconds() == 0, 1);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, vm = @VMReserved)]
    #[expected_failure(abort_code = 519)]
    fun update_global_time_post_genesis_vm_nil_proposer_increasing_timestamp(dr: signer, tc: signer, vm: signer) {
        Genesis::setup(&dr, &tc);
        assert(DiemTimestamp::now_microseconds() == 0, 0);
        DiemTimestamp::update_global_time(&vm, @VMReserved, 1);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, vm = @VMReserved)]
    #[expected_failure(abort_code = 519)]
    fun update_global_time_post_genesis_vm_not_nil_proposer_equal_timestamp(dr: signer, tc: signer, vm: signer) {
        Genesis::setup(&dr, &tc);
        assert(DiemTimestamp::now_microseconds() == 0, 0);
        DiemTimestamp::update_global_time(&vm, @0x1, 0);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, vm = @VMReserved)]
    fun update_global_time_post_genesis_vm_not_nil_proposer_increasing_timestamp(dr: signer, tc: signer, vm: signer) {
        Genesis::setup(&dr, &tc);
        assert(DiemTimestamp::now_microseconds() == 0, 0);
        DiemTimestamp::update_global_time(&vm, @0x1, 1);
        assert(DiemTimestamp::now_microseconds() == 1, 1);
    }

    #[test]
    #[expected_failure(abort_code = 257)]
    fun now_microseconds_pre_genesis() {
        DiemTimestamp::now_microseconds();
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    fun now_microseconds_post_genesis(dr: signer, tc: signer) {
        Genesis::setup(&dr, &tc);
        assert(DiemTimestamp::now_microseconds() == 0, 0);
    }

    #[test]
    #[expected_failure(abort_code = 257)]
    fun now_seconds_pre_genesis() {
        DiemTimestamp::now_seconds();
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    fun now_seconds_post_genesis(dr: signer, tc: signer) {
        Genesis::setup(&dr, &tc);
        assert(DiemTimestamp::now_seconds() == 0, 0);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    fun is_genesis(dr: signer, tc: signer) {
        assert(DiemTimestamp::is_genesis(), 0);
        DiemTimestamp::assert_genesis();
        Genesis::setup(&dr, &tc);
        assert(!DiemTimestamp::is_genesis(), 1);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    #[expected_failure(abort_code = 1)]
    fun assert_genesis(dr: signer, tc: signer) {
        Genesis::setup(&dr, &tc);
        DiemTimestamp::assert_genesis();
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    fun is_operating(dr: signer, tc: signer) {
        assert(!DiemTimestamp::is_operating(), 0);
        Genesis::setup(&dr, &tc);
        DiemTimestamp::assert_operating();
        assert(DiemTimestamp::is_operating(), 1);
    }

    #[test]
    #[expected_failure(abort_code = 257)]
    fun assert_operating() {
        DiemTimestamp::assert_operating();
    }
}

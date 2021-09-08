#[test_only]
module DiemFramework::RegisteredCurrencyTests {
    use DiemFramework::RegisteredCurrencies;
    use DiemFramework::Genesis;

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance, alice = @0x2)]
    #[expected_failure(abort_code = 1)]
    fun cannot_call_initialize_as_non_diem_root(dr: signer, tc: signer, alice: signer) {
        Genesis::setup(&dr, &tc);
        RegisteredCurrencies::initialize(&alice);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    #[expected_failure(abort_code = 1)]
    fun cannot_call_initialize_outside_genesis(dr: signer, tc: signer) {
        Genesis::setup(&dr, &tc);
        RegisteredCurrencies::initialize(&dr);
    }

    #[test(dr = @DiemRoot, tc = @TreasuryCompliance)]
    #[expected_failure(abort_code = 7)]
    fun cannot_add_currency_whose_currency_code_has_already_been_taken(dr: signer, tc: signer) {
        Genesis::setup(&dr, &tc);
        RegisteredCurrencies::add_currency_code(&dr, b"XDX");
    }
}

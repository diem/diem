script {
    use 0x1::VASP;
    /// Adds limits and an accounting window for `CoinType` currency to the parent VASP `account`.
    /// This transaction will fail if sent from a child account.
    fun allow_currency_for_vasp<CoinType>(account: &signer) {
        let _ = VASP::try_allow_currency<CoinType>(account)
    }
}

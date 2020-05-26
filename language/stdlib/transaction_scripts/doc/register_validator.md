
<a name="SCRIPT"></a>

# Script `register_validator.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, consensus_pubkey: vector&lt;u8&gt;, validator_network_signing_pubkey: vector&lt;u8&gt;, validator_network_identity_pubkey: vector&lt;u8&gt;, validator_network_address: vector&lt;u8&gt;, fullnodes_network_identity_pubkey: vector&lt;u8&gt;, fullnodes_network_address: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(
    account: &signer,
    consensus_pubkey: vector&lt;u8&gt;,
    validator_network_signing_pubkey: vector&lt;u8&gt;,
    validator_network_identity_pubkey: vector&lt;u8&gt;,
    validator_network_address: vector&lt;u8&gt;,
    fullnodes_network_identity_pubkey: vector&lt;u8&gt;,
    fullnodes_network_address: vector&lt;u8&gt;,
) {
  <a href="../../modules/doc/validator_config.md#0x0_ValidatorConfig_register_candidate_validator">ValidatorConfig::register_candidate_validator</a>(
      consensus_pubkey,
      validator_network_signing_pubkey,
      validator_network_identity_pubkey,
      validator_network_address,
      fullnodes_network_identity_pubkey,
      fullnodes_network_address
  );

  <b>let</b> sender = <a href="../../modules/doc/signer.md#0x0_Signer_address_of">Signer::address_of</a>(account);
  // Validating nodes need <b>to</b> accept all currencies in order <b>to</b> receive txn fees
  <b>if</b> (!<a href="../../modules/doc/libra_account.md#0x0_LibraAccount_accepts_currency">LibraAccount::accepts_currency</a>&lt;<a href="../../modules/doc/coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(sender)) {
      <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;<a href="../../modules/doc/coin1.md#0x0_Coin1_T">Coin1::T</a>&gt;(account)
  };
  <b>if</b> (!<a href="../../modules/doc/libra_account.md#0x0_LibraAccount_accepts_currency">LibraAccount::accepts_currency</a>&lt;<a href="../../modules/doc/coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(sender)) {
      <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;<a href="../../modules/doc/coin2.md#0x0_Coin2_T">Coin2::T</a>&gt;(account)
  };
  <b>if</b> (!<a href="../../modules/doc/libra_account.md#0x0_LibraAccount_accepts_currency">LibraAccount::accepts_currency</a>&lt;<a href="../../modules/doc/lbr.md#0x0_LBR_T">LBR::T</a>&gt;(sender)) {
      <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;<a href="../../modules/doc/lbr.md#0x0_LBR_T">LBR::T</a>&gt;(account)
  };
}
</code></pre>



</details>

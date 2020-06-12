
<a name="0x1_TransactionFee"></a>

# Module `0x1::TransactionFee`

### Table of Contents

-  [Struct `TransactionFeeCollection`](#0x1_TransactionFee_TransactionFeeCollection)
-  [Struct `TransactionFeePreburn`](#0x1_TransactionFee_TransactionFeePreburn)
-  [Function `initialize`](#0x1_TransactionFee_initialize)
-  [Function `add_txn_fee_currency`](#0x1_TransactionFee_add_txn_fee_currency)
-  [Function `preburn_fees`](#0x1_TransactionFee_preburn_fees)
-  [Function `burn_fees`](#0x1_TransactionFee_burn_fees)
-  [Function `preburn_coin`](#0x1_TransactionFee_preburn_coin)



<a name="0x1_TransactionFee_TransactionFeeCollection"></a>

## Struct `TransactionFeeCollection`

The
<code><a href="#0x1_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a></code> resource holds the
<code><a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a></code> for the
<code><a href="CoreAddresses.md#0x1_CoreAddresses_TRANSACTION_FEE_ADDRESS">CoreAddresses::TRANSACTION_FEE_ADDRESS</a>()</code> account.
This is used for the collection of the transaction fees since it
must be sent from the account at the
<code><a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>()</code> address.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TransactionFee_TransactionFeePreburn"></a>

## Struct `TransactionFeePreburn`

The
<code><a href="#0x1_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a></code> holds a preburn resource for each
fiat
<code>CoinType</code> that can be collected as a transaction fee.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>preburn: <a href="Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TransactionFee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect
transaction fees from the
<code>0xFEE</code> account with the
<code>0xB1E55ED</code> account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_initialize">initialize</a>(association: &signer, fee_account: &signer, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_initialize">initialize</a>(association: &signer, fee_account: &signer, auth_key_prefix: vector&lt;u8&gt;) {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association) == <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(),
        0
    );
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(fee_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_TRANSACTION_FEE_ADDRESS">CoreAddresses::TRANSACTION_FEE_ADDRESS</a>(),
        0
    );

    <a href="LibraAccount.md#0x1_LibraAccount_create_testnet_account">LibraAccount::create_testnet_account</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(
        association, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(fee_account), auth_key_prefix
    );
    // accept fees in all the currencies. No need <b>to</b> do this for <a href="LBR.md#0x1_LBR">LBR</a>
    <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(association, fee_account);
    <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(association, fee_account);

    <b>let</b> cap = <a href="LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(fee_account);
    move_to(fee_account, <a href="#0x1_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a> { cap });
}
</code></pre>



</details>

<a name="0x1_TransactionFee_add_txn_fee_currency"></a>

## Function `add_txn_fee_currency`

Sets ups the needed transaction fee state for a given
<code>CoinType</code> currency by
(1) configuring
<code>fee_account</code> to accept
<code>CoinType</code>
(2) publishing a wrapper of the
<code>Preburn&lt;CoinType&gt;</code> resource under
<code>fee_account</code>


<pre><code><b>fun</b> <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(association: &signer, fee_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(association: &signer, fee_account: &signer) {
    <a href="LibraAccount.md#0x1_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;CoinType&gt;(fee_account);
    move_to(fee_account, <a href="#0x1_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a>&lt;CoinType&gt; {
        preburn: <a href="Libra.md#0x1_Libra_create_preburn">Libra::create_preburn</a>(association)
    })
}
</code></pre>



</details>

<a name="0x1_TransactionFee_preburn_fees"></a>

## Function `preburn_fees`

Preburns the transaction fees collected in the
<code>CoinType</code> currency.
If the
<code>CoinType</code> is LBR, it unpacks the coin and preburns the
underlying fiat.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_preburn_fees">preburn_fees</a>&lt;CoinType&gt;(blessed_sender: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_preburn_fees">preburn_fees</a>&lt;CoinType&gt;(blessed_sender: &signer)
<b>acquires</b> <a href="#0x1_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a>, <a href="#0x1_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a> {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(blessed_sender) == <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>(),
        0
    );
    <b>if</b> (<a href="LBR.md#0x1_LBR_is_lbr">LBR::is_lbr</a>&lt;CoinType&gt;()) {
        <b>let</b> amount = <a href="LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_TRANSACTION_FEE_ADDRESS">CoreAddresses::TRANSACTION_FEE_ADDRESS</a>());
        <b>let</b> coins = <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">LibraAccount::withdraw_from</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(
            &borrow_global&lt;<a href="#0x1_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a>&gt;(0xFEE).cap,
            amount
        );
        <b>let</b> (coin1, coin2) = <a href="LBR.md#0x1_LBR_unpack">LBR::unpack</a>(blessed_sender, coins);
        <a href="#0x1_TransactionFee_preburn_coin">preburn_coin</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(coin1);
        <a href="#0x1_TransactionFee_preburn_coin">preburn_coin</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(coin2)
    } <b>else</b> {
        <b>let</b> amount = <a href="LibraAccount.md#0x1_LibraAccount_balance">LibraAccount::balance</a>&lt;CoinType&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_TRANSACTION_FEE_ADDRESS">CoreAddresses::TRANSACTION_FEE_ADDRESS</a>());
        <b>let</b> coins = <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">LibraAccount::withdraw_from</a>&lt;CoinType&gt;(
            &borrow_global&lt;<a href="#0x1_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a>&gt;(0xFEE).cap,
            amount
        );
        <a href="#0x1_TransactionFee_preburn_coin">preburn_coin</a>(coins)
    }
}
</code></pre>



</details>

<a name="0x1_TransactionFee_burn_fees"></a>

## Function `burn_fees`

Burns the already preburned fees from a previous call to
<code>preburn_fees</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_burn_fees">burn_fees</a>&lt;CoinType&gt;(burn_cap: &<a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionFee_burn_fees">burn_fees</a>&lt;CoinType&gt;(burn_cap: &BurnCapability&lt;CoinType&gt;)
<b>acquires</b> <a href="#0x1_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a> {
    <b>let</b> preburn = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a>&lt;CoinType&gt;&gt;(
        <a href="CoreAddresses.md#0x1_CoreAddresses_TRANSACTION_FEE_ADDRESS">CoreAddresses::TRANSACTION_FEE_ADDRESS</a>()
    ).preburn;
    <a href="Libra.md#0x1_Libra_burn_with_resource_cap">Libra::burn_with_resource_cap</a>(
        preburn,
        <a href="CoreAddresses.md#0x1_CoreAddresses_TRANSACTION_FEE_ADDRESS">CoreAddresses::TRANSACTION_FEE_ADDRESS</a>(),
        burn_cap
    )
}
</code></pre>



</details>

<a name="0x1_TransactionFee_preburn_coin"></a>

## Function `preburn_coin`



<pre><code><b>fun</b> <a href="#0x1_TransactionFee_preburn_coin">preburn_coin</a>&lt;CoinType&gt;(coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_TransactionFee_preburn_coin">preburn_coin</a>&lt;CoinType&gt;(coin: <a href="Libra.md#0x1_Libra">Libra</a>&lt;CoinType&gt;)
<b>acquires</b> <a href="#0x1_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a> {
    <b>let</b> preburn = &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a>&lt;CoinType&gt;&gt;(
        <a href="CoreAddresses.md#0x1_CoreAddresses_TRANSACTION_FEE_ADDRESS">CoreAddresses::TRANSACTION_FEE_ADDRESS</a>()
    ).preburn;
    <a href="Libra.md#0x1_Libra_preburn_with_resource">Libra::preburn_with_resource</a>(
        coin,
        preburn,
        <a href="CoreAddresses.md#0x1_CoreAddresses_TRANSACTION_FEE_ADDRESS">CoreAddresses::TRANSACTION_FEE_ADDRESS</a>()
    );
}
</code></pre>



</details>

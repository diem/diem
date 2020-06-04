
<a name="0x0_TransactionFee"></a>

# Module `0x0::TransactionFee`

### Table of Contents

-  [Struct `TransactionFeeCollection`](#0x0_TransactionFee_TransactionFeeCollection)
-  [Struct `TransactionFeePreburn`](#0x0_TransactionFee_TransactionFeePreburn)
-  [Struct `LBRIdent`](#0x0_TransactionFee_LBRIdent)
-  [Function `initialize`](#0x0_TransactionFee_initialize)
-  [Function `add_txn_fee_currency`](#0x0_TransactionFee_add_txn_fee_currency)
-  [Function `is_lbr`](#0x0_TransactionFee_is_lbr)
-  [Function `preburn_fees`](#0x0_TransactionFee_preburn_fees)
-  [Function `burn_fees`](#0x0_TransactionFee_burn_fees)
-  [Function `preburn_coin`](#0x0_TransactionFee_preburn_coin)



<a name="0x0_TransactionFee_TransactionFeeCollection"></a>

## Struct `TransactionFeeCollection`

The
<code><a href="#0x0_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a></code> resource holds the
<code><a href="LibraAccount.md#0x0_LibraAccount_withdraw_with_capability">LibraAccount::withdraw_with_capability</a></code> for the
<code>0xFEE</code> account.
This is used for the collection of the transaction fees since it
must be sent from the account at the
<code>0xB1E55ED</code> address.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>cap: <a href="LibraAccount.md#0x0_LibraAccount_WithdrawalCapability">LibraAccount::WithdrawalCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_TransactionFee_TransactionFeePreburn"></a>

## Struct `TransactionFeePreburn`

The
<code><a href="#0x0_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a></code> holds a preburn resource for each
fiat
<code>CoinType</code> that can be collected as a transaction fee.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>preburn: <a href="Libra.md#0x0_Libra_Preburn">Libra::Preburn</a>&lt;CoinType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_TransactionFee_LBRIdent"></a>

## Struct `LBRIdent`

We need to be able to determine if
<code>CoinType</code> is LBR or not in
order to unpack it properly before burning it. This resource is
instantiated with
<code><a href="LBR.md#0x0_LBR">LBR</a></code> and published in
<code><a href="#0x0_TransactionFee_initialize">TransactionFee::initialize</a></code>.
We then use this to determine if the /
<code>CoinType</code> is LBR in
<code><a href="#0x0_TransactionFee_is_lbr">TransactionFee::is_lbr</a></code>.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_TransactionFee_LBRIdent">LBRIdent</a>&lt;CoinType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_TransactionFee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect
transaction fees by the
<code>0xB1E55ED</code> account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_initialize">initialize</a>(blessed_account: &signer, fee_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_initialize">initialize</a>(blessed_account: &signer, fee_account: &signer) {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(blessed_account) == 0xB1E55ED, 0);
    <b>let</b> cap = <a href="LibraAccount.md#0x0_LibraAccount_extract_sender_withdrawal_capability">LibraAccount::extract_sender_withdrawal_capability</a>(fee_account);
    move_to(blessed_account, <a href="#0x0_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a> { cap });
    move_to(blessed_account, <a href="#0x0_TransactionFee_LBRIdent">LBRIdent</a>&lt;<a href="LBR.md#0x0_LBR">LBR</a>&gt;{})
}
</code></pre>



</details>

<a name="0x0_TransactionFee_add_txn_fee_currency"></a>

## Function `add_txn_fee_currency`

Sets ups the needed transaction fee state for a given
<code>CoinType</code>
currency.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(fee_account: &signer, burn_cap: &<a href="Libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_add_txn_fee_currency">add_txn_fee_currency</a>&lt;CoinType&gt;(fee_account: &signer, burn_cap: &BurnCapability&lt;CoinType&gt;) {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(fee_account) == 0xFEE, 0);
    <a href="LibraAccount.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;CoinType&gt;(fee_account);
    move_to(fee_account, <a href="#0x0_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a>&lt;CoinType&gt;{
        preburn: <a href="Libra.md#0x0_Libra_new_preburn_with_capability">Libra::new_preburn_with_capability</a>(burn_cap)
    })
}
</code></pre>



</details>

<a name="0x0_TransactionFee_is_lbr"></a>

## Function `is_lbr`

Returns whether
<code>CoinType</code> is LBR or not. This is needed since we
will need to unpack LBR before burning it when collecting the
transaction fees.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_is_lbr">is_lbr</a>&lt;CoinType&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_is_lbr">is_lbr</a>&lt;CoinType&gt;(): bool {
    exists&lt;<a href="#0x0_TransactionFee_LBRIdent">LBRIdent</a>&lt;CoinType&gt;&gt;(0xB1E55ED)
}
</code></pre>



</details>

<a name="0x0_TransactionFee_preburn_fees"></a>

## Function `preburn_fees`

Preburns the transaction fees collected in the
<code>CoinType</code> currency.
If the
<code>CoinType</code> is LBR, it unpacks the coin and preburns the
underlying fiat.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_preburn_fees">preburn_fees</a>&lt;CoinType&gt;(blessed_sender: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_preburn_fees">preburn_fees</a>&lt;CoinType&gt;(blessed_sender: &signer)
<b>acquires</b> <a href="#0x0_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a>, <a href="#0x0_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a> {
    <b>if</b> (<a href="#0x0_TransactionFee_is_lbr">is_lbr</a>&lt;CoinType&gt;()) {
        <b>let</b> amount = <a href="LibraAccount.md#0x0_LibraAccount_balance">LibraAccount::balance</a>&lt;<a href="LBR.md#0x0_LBR">LBR</a>&gt;(0xFEE);
        <b>let</b> coins = <a href="LibraAccount.md#0x0_LibraAccount_withdraw_with_capability">LibraAccount::withdraw_with_capability</a>&lt;<a href="LBR.md#0x0_LBR">LBR</a>&gt;(
            &borrow_global&lt;<a href="#0x0_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a>&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(blessed_sender)).cap,
            amount
        );
        <b>let</b> (coin1, coin2) = <a href="LBR.md#0x0_LBR_unpack">LBR::unpack</a>(coins);
        <a href="#0x0_TransactionFee_preburn_coin">preburn_coin</a>&lt;<a href="Coin1.md#0x0_Coin1">Coin1</a>&gt;(coin1);
        <a href="#0x0_TransactionFee_preburn_coin">preburn_coin</a>&lt;<a href="Coin2.md#0x0_Coin2">Coin2</a>&gt;(coin2)
    } <b>else</b> {
        <b>let</b> amount = <a href="LibraAccount.md#0x0_LibraAccount_balance">LibraAccount::balance</a>&lt;CoinType&gt;(0xFEE);
        <b>let</b> coins = <a href="LibraAccount.md#0x0_LibraAccount_withdraw_with_capability">LibraAccount::withdraw_with_capability</a>&lt;CoinType&gt;(
            &borrow_global&lt;<a href="#0x0_TransactionFee_TransactionFeeCollection">TransactionFeeCollection</a>&gt;(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(blessed_sender)).cap,
            amount
        );
        <a href="#0x0_TransactionFee_preburn_coin">preburn_coin</a>(coins)
    }
}
</code></pre>



</details>

<a name="0x0_TransactionFee_burn_fees"></a>

## Function `burn_fees`

Burns the already preburned fees from a previous call to
<code>preburn_fees</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_burn_fees">burn_fees</a>&lt;CoinType&gt;(blessed_account: &signer, burn_cap: &<a href="Libra.md#0x0_Libra_BurnCapability">Libra::BurnCapability</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_burn_fees">burn_fees</a>&lt;CoinType&gt;(blessed_account: &signer, burn_cap: &BurnCapability&lt;CoinType&gt;)
<b>acquires</b> <a href="#0x0_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a> {
    Transaction::assert(<a href="Signer.md#0x0_Signer_address_of">Signer::address_of</a>(blessed_account) == 0xB1E55ED, 0);
    <b>let</b> preburn = &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a>&lt;CoinType&gt;&gt;(0xFEE).preburn;
    <a href="Libra.md#0x0_Libra_burn_with_resource_cap">Libra::burn_with_resource_cap</a>(
        preburn,
        0xFEE,
        burn_cap
    )
}
</code></pre>



</details>

<a name="0x0_TransactionFee_preburn_coin"></a>

## Function `preburn_coin`



<pre><code><b>fun</b> <a href="#0x0_TransactionFee_preburn_coin">preburn_coin</a>&lt;CoinType&gt;(coin: <a href="Libra.md#0x0_Libra_T">Libra::T</a>&lt;CoinType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_TransactionFee_preburn_coin">preburn_coin</a>&lt;CoinType&gt;(coin: <a href="Libra.md#0x0_Libra">Libra</a>&lt;CoinType&gt;)
<b>acquires</b> <a href="#0x0_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a> {
    <b>let</b> preburn = &<b>mut</b> borrow_global_mut&lt;<a href="#0x0_TransactionFee_TransactionFeePreburn">TransactionFeePreburn</a>&lt;CoinType&gt;&gt;(0xFEE).preburn;
    <a href="Libra.md#0x0_Libra_preburn_with_resource">Libra::preburn_with_resource</a>(
        coin,
        preburn,
        0xFEE
    );
}
</code></pre>



</details>

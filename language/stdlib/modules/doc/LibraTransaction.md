
<a name="0x1_LibraTransaction"></a>

# Module `0x1::LibraTransaction`

### Table of Contents

-  [Resource `LibraWriteSetManager`](#0x1_LibraTransaction_LibraWriteSetManager)
-  [Struct `UpgradeEvent`](#0x1_LibraTransaction_UpgradeEvent)
-  [Const `MAX_U64`](#0x1_LibraTransaction_MAX_U64)
-  [Const `ELIBRA_WRITE_SET_MANAGER`](#0x1_LibraTransaction_ELIBRA_WRITE_SET_MANAGER)
-  [Const `PROLOGUE_ETRANSACTION_EXPIRED`](#0x1_LibraTransaction_PROLOGUE_ETRANSACTION_EXPIRED)
-  [Const `PROLOGUE_EBAD_CHAIN_ID`](#0x1_LibraTransaction_PROLOGUE_EBAD_CHAIN_ID)
-  [Const `PROLOGUE_ESCRIPT_NOT_ALLOWED`](#0x1_LibraTransaction_PROLOGUE_ESCRIPT_NOT_ALLOWED)
-  [Const `PROLOGUE_EMODULE_NOT_ALLOWED`](#0x1_LibraTransaction_PROLOGUE_EMODULE_NOT_ALLOWED)
-  [Const `EGAS`](#0x1_LibraTransaction_EGAS)
-  [Function `initialize`](#0x1_LibraTransaction_initialize)
-  [Function `module_prologue`](#0x1_LibraTransaction_module_prologue)
-  [Function `script_prologue`](#0x1_LibraTransaction_script_prologue)
-  [Function `prologue_common`](#0x1_LibraTransaction_prologue_common)
-  [Function `writeset_prologue`](#0x1_LibraTransaction_writeset_prologue)
-  [Function `success_epilogue`](#0x1_LibraTransaction_success_epilogue)
-  [Function `failure_epilogue`](#0x1_LibraTransaction_failure_epilogue)
-  [Function `writeset_epilogue`](#0x1_LibraTransaction_writeset_epilogue)
-  [Specification](#0x1_LibraTransaction_Specification)
    -  [Function `initialize`](#0x1_LibraTransaction_Specification_initialize)



<a name="0x1_LibraTransaction_LibraWriteSetManager"></a>

## Resource `LibraWriteSetManager`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraTransaction_LibraWriteSetManager">LibraWriteSetManager</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>upgrade_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraTransaction_UpgradeEvent">LibraTransaction::UpgradeEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraTransaction_UpgradeEvent"></a>

## Struct `UpgradeEvent`



<pre><code><b>struct</b> <a href="#0x1_LibraTransaction_UpgradeEvent">UpgradeEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>writeset_payload: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraTransaction_MAX_U64"></a>

## Const `MAX_U64`



<pre><code><b>const</b> MAX_U64: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_LibraTransaction_ELIBRA_WRITE_SET_MANAGER"></a>

## Const `ELIBRA_WRITE_SET_MANAGER`

The
<code><a href="#0x1_LibraTransaction_LibraWriteSetManager">LibraWriteSetManager</a></code> was not in the required state


<pre><code><b>const</b> ELIBRA_WRITE_SET_MANAGER: u64 = 0;
</code></pre>



<a name="0x1_LibraTransaction_PROLOGUE_ETRANSACTION_EXPIRED"></a>

## Const `PROLOGUE_ETRANSACTION_EXPIRED`



<pre><code><b>const</b> PROLOGUE_ETRANSACTION_EXPIRED: u64 = 6;
</code></pre>



<a name="0x1_LibraTransaction_PROLOGUE_EBAD_CHAIN_ID"></a>

## Const `PROLOGUE_EBAD_CHAIN_ID`



<pre><code><b>const</b> PROLOGUE_EBAD_CHAIN_ID: u64 = 7;
</code></pre>



<a name="0x1_LibraTransaction_PROLOGUE_ESCRIPT_NOT_ALLOWED"></a>

## Const `PROLOGUE_ESCRIPT_NOT_ALLOWED`



<pre><code><b>const</b> PROLOGUE_ESCRIPT_NOT_ALLOWED: u64 = 8;
</code></pre>



<a name="0x1_LibraTransaction_PROLOGUE_EMODULE_NOT_ALLOWED"></a>

## Const `PROLOGUE_EMODULE_NOT_ALLOWED`



<pre><code><b>const</b> PROLOGUE_EMODULE_NOT_ALLOWED: u64 = 9;
</code></pre>



<a name="0x1_LibraTransaction_EGAS"></a>

## Const `EGAS`

An invalid amount of gas units was provided for execution of the transaction


<pre><code><b>const</b> EGAS: u64 = 20;
</code></pre>



<a name="0x1_LibraTransaction_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransaction_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransaction_initialize">initialize</a>(account: &signer) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    // Operational constraint
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(account);

    <b>assert</b>(
        !exists&lt;<a href="#0x1_LibraTransaction_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(ELIBRA_WRITE_SET_MANAGER)
    );
    move_to(
        account,
        <a href="#0x1_LibraTransaction_LibraWriteSetManager">LibraWriteSetManager</a> {
            upgrade_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_LibraTransaction_UpgradeEvent">Self::UpgradeEvent</a>&gt;(account),
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraTransaction_module_prologue"></a>

## Function `module_prologue`

The prologue for module transaction


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_module_prologue">module_prologue</a>&lt;Token&gt;(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_module_prologue">module_prologue</a>&lt;Token&gt;(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
    chain_id: u8,
) {
    <b>assert</b>(
        <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_module_allowed">LibraTransactionPublishingOption::is_module_allowed</a>(sender),
        PROLOGUE_EMODULE_NOT_ALLOWED
    );

    <a href="#0x1_LibraTransaction_prologue_common">prologue_common</a>&lt;Token&gt;(
        sender,
        txn_sequence_number,
        txn_public_key,
        txn_gas_price,
        txn_max_gas_units,
        txn_expiration_time,
        chain_id,
    )
}
</code></pre>



</details>

<a name="0x1_LibraTransaction_script_prologue"></a>

## Function `script_prologue`

The prologue for script transaction


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_script_prologue">script_prologue</a>&lt;Token&gt;(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, chain_id: u8, script_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_script_prologue">script_prologue</a>&lt;Token&gt;(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
    chain_id: u8,
    script_hash: vector&lt;u8&gt;,
) {
    <b>assert</b>(
        <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_script_allowed">LibraTransactionPublishingOption::is_script_allowed</a>(sender, &script_hash),
        PROLOGUE_ESCRIPT_NOT_ALLOWED
    );

    <a href="#0x1_LibraTransaction_prologue_common">prologue_common</a>&lt;Token&gt;(
        sender,
        txn_sequence_number,
        txn_public_key,
        txn_gas_price,
        txn_max_gas_units,
        txn_expiration_time,
        chain_id,
    )
}
</code></pre>



</details>

<a name="0x1_LibraTransaction_prologue_common"></a>

## Function `prologue_common`

The common prologue is invoked at the beginning of every transaction
It verifies:
- The account's auth key matches the transaction's public key
- That the account has enough balance to pay for all of the gas
- That the sequence number matches the transaction's sequence key


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_prologue_common">prologue_common</a>&lt;Token&gt;(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time_seconds: u64, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_prologue_common">prologue_common</a>&lt;Token&gt;(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time_seconds: u64,
    chain_id: u8,
) {
    // Check that the chain ID stored on-chain matches the chain ID specified by the transaction
    <b>assert</b>(<a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() == chain_id, PROLOGUE_EBAD_CHAIN_ID);

    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
    <a href="LibraAccount.md#0x1_LibraAccount_check_sender">LibraAccount::check_sender</a>&lt;Token&gt;(sender, max_transaction_fee, txn_sequence_number, txn_public_key);

    <b>assert</b>(
        <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_seconds">LibraTimestamp::now_seconds</a>() &lt; txn_expiration_time_seconds,
        PROLOGUE_ETRANSACTION_EXPIRED
    );
}
</code></pre>



</details>

<a name="0x1_LibraTransaction_writeset_prologue"></a>

## Function `writeset_prologue`



<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_writeset_prologue">writeset_prologue</a>(account: &signer, writeset_sequence_number: u64, writeset_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_writeset_prologue">writeset_prologue</a>(
    account: &signer,
    writeset_sequence_number: u64,
    writeset_public_key: vector&lt;u8&gt;,
) {
    // Currency code don't matter here <b>as</b> it won't be charged anyway.
    <a href="LibraAccount.md#0x1_LibraAccount_check_sender">LibraAccount::check_sender</a>&lt;<a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;(account, 0, writeset_sequence_number, writeset_public_key);
    // TODO: Check ChainID & Expiration Time?
}
</code></pre>



</details>

<a name="0x1_LibraTransaction_success_epilogue"></a>

## Function `success_epilogue`

The success_epilogue is invoked at the end of successfully executed transactions.


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_success_epilogue">success_epilogue</a>&lt;Token&gt;(account: &signer, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_success_epilogue">success_epilogue</a>&lt;Token&gt;(
    account: &signer,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64
) {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);

    // Charge for gas
    <b>assert</b>(txn_max_gas_units &gt;= gas_units_remaining, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(EGAS));
    <b>let</b> gas_used = txn_max_gas_units - gas_units_remaining;
    <b>assert</b>((txn_gas_price <b>as</b> u128) * (gas_used <b>as</b> u128) &lt;= MAX_U64, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(EGAS));
    <b>let</b> transaction_fee_amount = txn_gas_price * gas_used;

    <a href="LibraAccount.md#0x1_LibraAccount_charge_transaction">LibraAccount::charge_transaction</a>&lt;Token&gt;(sender, transaction_fee_amount, txn_sequence_number);
}
</code></pre>



</details>

<a name="0x1_LibraTransaction_failure_epilogue"></a>

## Function `failure_epilogue`

The failure_epilogue is invoked at the end of transactions when the transaction is aborted during execution or
during
<code>success_epilogue</code>.


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_failure_epilogue">failure_epilogue</a>&lt;Token&gt;(account: &signer, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_failure_epilogue">failure_epilogue</a>&lt;Token&gt;(
    account: &signer,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64
) {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Charge for gas
    <b>assert</b>(txn_max_gas_units &gt;= gas_units_remaining, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(EGAS));
    <b>let</b> gas_used = txn_max_gas_units - gas_units_remaining;
    <b>assert</b>((txn_gas_price <b>as</b> u128) * (gas_used <b>as</b> u128) &lt;= MAX_U64, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(EGAS));
    <b>let</b> transaction_fee_amount = txn_gas_price * gas_used;

    <a href="LibraAccount.md#0x1_LibraAccount_charge_transaction">LibraAccount::charge_transaction</a>&lt;Token&gt;(sender, transaction_fee_amount, txn_sequence_number);
}
</code></pre>



</details>

<a name="0x1_LibraTransaction_writeset_epilogue"></a>

## Function `writeset_epilogue`



<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_writeset_epilogue">writeset_epilogue</a>(lr_account: &signer, writeset_payload: vector&lt;u8&gt;, writeset_sequence_number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraTransaction_writeset_epilogue">writeset_epilogue</a>(
    lr_account: &signer,
    writeset_payload: vector&lt;u8&gt;,
    writeset_sequence_number: u64
) <b>acquires</b> <a href="#0x1_LibraTransaction_LibraWriteSetManager">LibraWriteSetManager</a> {
    <b>let</b> t_ref = borrow_global_mut&lt;<a href="#0x1_LibraTransaction_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraTransaction_UpgradeEvent">Self::UpgradeEvent</a>&gt;(
        &<b>mut</b> t_ref.upgrade_events,
        <a href="#0x1_LibraTransaction_UpgradeEvent">UpgradeEvent</a> { writeset_payload },
    );
    // Currency code don't matter here <b>as</b> it won't be charged anyway.
    <a href="LibraAccount.md#0x1_LibraAccount_charge_transaction">LibraAccount::charge_transaction</a>&lt;<a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account), 0, writeset_sequence_number);
    <a href="LibraConfig.md#0x1_LibraConfig_reconfigure">LibraConfig::reconfigure</a>(lr_account)
}
</code></pre>



</details>

<a name="0x1_LibraTransaction_Specification"></a>

## Specification



<pre><code><b>invariant</b> [<b>global</b>]
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; exists&lt;<a href="#0x1_LibraTransaction_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>



<a name="0x1_LibraTransaction_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransaction_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>;
<b>aborts_if</b> exists&lt;<a href="#0x1_LibraTransaction_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) with Errors::ALREADY_PUBLISHED;
</code></pre>

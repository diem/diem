
<a name="0x1_LibraAccount"></a>

# Module `0x1::LibraAccount`

### Table of Contents

-  [Resource `LibraAccount`](#0x1_LibraAccount_LibraAccount)
-  [Resource `Balance`](#0x1_LibraAccount_Balance)
-  [Resource `WithdrawCapability`](#0x1_LibraAccount_WithdrawCapability)
-  [Resource `KeyRotationCapability`](#0x1_LibraAccount_KeyRotationCapability)
-  [Resource `AccountOperationsCapability`](#0x1_LibraAccount_AccountOperationsCapability)
-  [Struct `SentPaymentEvent`](#0x1_LibraAccount_SentPaymentEvent)
-  [Struct `ReceivedPaymentEvent`](#0x1_LibraAccount_ReceivedPaymentEvent)
-  [Function `initialize`](#0x1_LibraAccount_initialize)
-  [Function `has_published_account_limits`](#0x1_LibraAccount_has_published_account_limits)
-  [Function `should_track_limits_for_account`](#0x1_LibraAccount_should_track_limits_for_account)
-  [Function `staple_lbr`](#0x1_LibraAccount_staple_lbr)
-  [Function `unstaple_lbr`](#0x1_LibraAccount_unstaple_lbr)
-  [Function `deposit`](#0x1_LibraAccount_deposit)
-  [Function `tiered_mint`](#0x1_LibraAccount_tiered_mint)
-  [Function `cancel_burn`](#0x1_LibraAccount_cancel_burn)
-  [Function `withdraw_from_balance`](#0x1_LibraAccount_withdraw_from_balance)
-  [Function `withdraw_from`](#0x1_LibraAccount_withdraw_from)
-  [Function `preburn`](#0x1_LibraAccount_preburn)
-  [Function `extract_withdraw_capability`](#0x1_LibraAccount_extract_withdraw_capability)
-  [Function `restore_withdraw_capability`](#0x1_LibraAccount_restore_withdraw_capability)
-  [Function `pay_from`](#0x1_LibraAccount_pay_from)
-  [Function `rotate_authentication_key`](#0x1_LibraAccount_rotate_authentication_key)
-  [Function `extract_key_rotation_capability`](#0x1_LibraAccount_extract_key_rotation_capability)
-  [Function `restore_key_rotation_capability`](#0x1_LibraAccount_restore_key_rotation_capability)
-  [Function `add_currencies_for_account`](#0x1_LibraAccount_add_currencies_for_account)
-  [Function `make_account`](#0x1_LibraAccount_make_account)
-  [Function `create_libra_root_account`](#0x1_LibraAccount_create_libra_root_account)
-  [Function `create_treasury_compliance_account`](#0x1_LibraAccount_create_treasury_compliance_account)
-  [Function `create_designated_dealer`](#0x1_LibraAccount_create_designated_dealer)
-  [Function `create_parent_vasp_account`](#0x1_LibraAccount_create_parent_vasp_account)
-  [Function `create_child_vasp_account`](#0x1_LibraAccount_create_child_vasp_account)
-  [Function `create_signer`](#0x1_LibraAccount_create_signer)
-  [Function `destroy_signer`](#0x1_LibraAccount_destroy_signer)
-  [Function `balance_for`](#0x1_LibraAccount_balance_for)
-  [Function `balance`](#0x1_LibraAccount_balance)
-  [Function `add_currency`](#0x1_LibraAccount_add_currency)
-  [Function `accepts_currency`](#0x1_LibraAccount_accepts_currency)
-  [Function `sequence_number_for_account`](#0x1_LibraAccount_sequence_number_for_account)
-  [Function `sequence_number`](#0x1_LibraAccount_sequence_number)
-  [Function `authentication_key`](#0x1_LibraAccount_authentication_key)
-  [Function `delegated_key_rotation_capability`](#0x1_LibraAccount_delegated_key_rotation_capability)
-  [Function `delegated_withdraw_capability`](#0x1_LibraAccount_delegated_withdraw_capability)
-  [Function `withdraw_capability_address`](#0x1_LibraAccount_withdraw_capability_address)
-  [Function `key_rotation_capability_address`](#0x1_LibraAccount_key_rotation_capability_address)
-  [Function `exists_at`](#0x1_LibraAccount_exists_at)
-  [Function `prologue`](#0x1_LibraAccount_prologue)
-  [Function `epilogue`](#0x1_LibraAccount_epilogue)
-  [Function `success_epilogue`](#0x1_LibraAccount_success_epilogue)
-  [Function `failure_epilogue`](#0x1_LibraAccount_failure_epilogue)
-  [Function `bump_sequence_number`](#0x1_LibraAccount_bump_sequence_number)
-  [Function `create_validator_account`](#0x1_LibraAccount_create_validator_account)
-  [Function `create_validator_operator_account`](#0x1_LibraAccount_create_validator_operator_account)
-  [Specification](#0x1_LibraAccount_Specification)
    -  [Function `should_track_limits_for_account`](#0x1_LibraAccount_Specification_should_track_limits_for_account)
    -  [Function `deposit`](#0x1_LibraAccount_Specification_deposit)
    -  [Function `withdraw_from_balance`](#0x1_LibraAccount_Specification_withdraw_from_balance)
    -  [Function `rotate_authentication_key`](#0x1_LibraAccount_Specification_rotate_authentication_key)
    -  [Function `extract_key_rotation_capability`](#0x1_LibraAccount_Specification_extract_key_rotation_capability)
    -  [Function `restore_key_rotation_capability`](#0x1_LibraAccount_Specification_restore_key_rotation_capability)
    -  [Function `create_designated_dealer`](#0x1_LibraAccount_Specification_create_designated_dealer)



<a name="0x1_LibraAccount_LibraAccount"></a>

## Resource `LibraAccount`

Every Libra account has a LibraAccount resource


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount">LibraAccount</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>authentication_key: vector&lt;u8&gt;</code>
</dt>
<dd>
 The current authentication key.
 This can be different than the key used to create the account
</dd>
<dt>

<code>withdrawal_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>&gt;</code>
</dt>
<dd>
 A
<code>withdrawal_capability</code> allows whoever holds this capability
 to withdraw from the account. At the time of account creation
 this capability is stored in this option. It can later be
 and can also be restored via
<code>restore_withdraw_capability</code>.
</dd>
<dt>

<code>key_rotation_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>&gt;</code>
</dt>
<dd>
 A
<code>key_rotation_capability</code> allows whoever holds this capability
 the ability to rotate the authentication key for the account. At
 the time of account creation this capability is stored in this
 option. It can later be "extracted" from this field via
 <code>extract_key_rotation_capability</code>, and can also be restored via
 <code>restore_key_rotation_capability</code>.
</dd>
<dt>

<code>received_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a>&gt;</code>
</dt>
<dd>
 Event handle for received event
</dd>
<dt>

<code>sent_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a>&gt;</code>
</dt>
<dd>
 Event handle for sent event
</dd>
<dt>

<code>sequence_number: u64</code>
</dt>
<dd>
 The current sequence number.
 Incremented by one each time a transaction is submitted
</dd>
</dl>


</details>

<a name="0x1_LibraAccount_Balance"></a>

## Resource `Balance`

A resource that holds the coins stored in this account


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_WithdrawCapability"></a>

## Resource `WithdrawCapability`

The holder of WithdrawCapability for account_address can withdraw Libra from
account_address/LibraAccount/balance.
There is at most one WithdrawCapability in existence for a given address.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>account_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_KeyRotationCapability"></a>

## Resource `KeyRotationCapability`

The holder of KeyRotationCapability for account_address can rotate the authentication key for
account_address (i.e., write to account_address/LibraAccount/authentication_key).
There is at most one KeyRotationCapability in existence for a given address.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>account_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_AccountOperationsCapability"></a>

## Resource `AccountOperationsCapability`

A wrapper around an
<code>AccountLimitMutationCapability</code> which is used to check for account limits
and to record freeze/unfreeze events.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>limits_cap: <a href="AccountLimits.md#0x1_AccountLimits_AccountLimitMutationCapability">AccountLimits::AccountLimitMutationCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_SentPaymentEvent"></a>

## Struct `SentPaymentEvent`

Message for sent events


<pre><code><b>struct</b> <a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>
 The amount of Libra<Token> sent
</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 The code symbol for the currency that was sent
</dd>
<dt>

<code>payee: address</code>
</dt>
<dd>
 The address that was paid
</dd>
<dt>

<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>
 Metadata associated with the payment
</dd>
</dl>


</details>

<a name="0x1_LibraAccount_ReceivedPaymentEvent"></a>

## Struct `ReceivedPaymentEvent`

Message for received events


<pre><code><b>struct</b> <a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u64</code>
</dt>
<dd>
 The amount of Libra<Token> received
</dd>
<dt>

<code>currency_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 The code symbol for the currency that was received
</dd>
<dt>

<code>payer: address</code>
</dt>
<dd>
 The address that sent the coin
</dd>
<dt>

<code>metadata: vector&lt;u8&gt;</code>
</dt>
<dd>
 Metadata associated with the payment
</dd>
</dl>


</details>

<a name="0x1_LibraAccount_initialize"></a>

## Function `initialize`

Initialize this module. This is only callable from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_initialize">initialize</a>(
    lr_account: &signer,
) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    // Operational constraint, not a privilege constraint.
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(lr_account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EINVALID_SINGLETON_ADDRESS);
    move_to(
        lr_account,
        <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
            limits_cap: <a href="AccountLimits.md#0x1_AccountLimits_grant_mutation_capability">AccountLimits::grant_mutation_capability</a>(lr_account),
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_has_published_account_limits"></a>

## Function `has_published_account_limits`

Return
<code><b>true</b></code> if
<code>addr</code> has already published account limits for
<code>Token</code>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_has_published_account_limits">has_published_account_limits</a>&lt;Token&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_has_published_account_limits">has_published_account_limits</a>&lt;Token&gt;(addr: address): bool {
    <b>if</b> (<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(addr)) <a href="VASP.md#0x1_VASP_has_account_limits">VASP::has_account_limits</a>&lt;Token&gt;(addr)
    <b>else</b> <a href="AccountLimits.md#0x1_AccountLimits_has_window_published">AccountLimits::has_window_published</a>&lt;Token&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_should_track_limits_for_account"></a>

## Function `should_track_limits_for_account`

Returns whether we should track and record limits for the
<code>payer</code> or
<code>payee</code> account.
Depending on the
<code>is_withdrawal</code> flag passed in we determine whether the
<code>payer</code> or
<code>payee</code> account is being queried.
<code><a href="VASP.md#0x1_VASP">VASP</a>-&gt;any</code> and
<code>any-&gt;<a href="VASP.md#0x1_VASP">VASP</a></code> transfers are tracked in the VASP.


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(payer: address, payee: address, is_withdrawal: bool): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(
    payer: address, payee: address, is_withdrawal: bool
): bool {
    <b>if</b> (is_withdrawal) {
        <a href="#0x1_LibraAccount_has_published_account_limits">has_published_account_limits</a>&lt;Token&gt;(payer) &&
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payer) &&
        (!<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payee) || !<a href="VASP.md#0x1_VASP_is_same_vasp">VASP::is_same_vasp</a>(payer, payee))
    } <b>else</b> {
        <a href="#0x1_LibraAccount_has_published_account_limits">has_published_account_limits</a>&lt;Token&gt;(payee) &&
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payee) &&
        (!<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payer) || !<a href="VASP.md#0x1_VASP_is_same_vasp">VASP::is_same_vasp</a>(payee, payer))
    }
}
</code></pre>



</details>

<a name="0x1_LibraAccount_staple_lbr"></a>

## Function `staple_lbr`

Use
<code>cap</code> to mint
<code>amount_lbr</code> LBR by withdrawing the appropriate quantity of reserve assets
from
<code>cap.address</code>, giving them to the LBR reserve, and depositing the LBR into
<code>cap.address</code>.
The
<code>payee</code> address in the
<code><a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a></code>s emitted by this function is the LBR reserve
address to signify that this was a special payment that debits the
<code>cap.addr</code>'s balance and
credits the LBR reserve.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_staple_lbr">staple_lbr</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_staple_lbr">staple_lbr</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>, amount_lbr: u64)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> cap_address = cap.account_address;
    // <b>use</b> the <a href="LBR.md#0x1_LBR">LBR</a> reserve address <b>as</b> `payee_address`
    <b>let</b> payee_address = <a href="LBR.md#0x1_LBR_reserve_address">LBR::reserve_address</a>();
    <b>let</b> (amount_coin1, amount_coin2) = <a href="LBR.md#0x1_LBR_calculate_component_amounts_for_lbr">LBR::calculate_component_amounts_for_lbr</a>(amount_lbr);
    <b>let</b> coin1 = <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(cap, payee_address, amount_coin1, x"");
    <b>let</b> coin2 = <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(cap, payee_address, amount_coin2, x"");
    // Create `amount_lbr` <a href="LBR.md#0x1_LBR">LBR</a>
    <b>let</b> lbr = <a href="LBR.md#0x1_LBR_create">LBR::create</a>(amount_lbr, coin1, coin2);
    // <b>use</b> the reserved address <b>as</b> the payer for the <a href="LBR.md#0x1_LBR">LBR</a> payment because the funds did not come
    // from an existing balance
    <a href="#0x1_LibraAccount_deposit">deposit</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), cap_address, lbr, x"", x"");
}
</code></pre>



</details>

<a name="0x1_LibraAccount_unstaple_lbr"></a>

## Function `unstaple_lbr`

Use
<code>cap</code> to withdraw
<code>amount_lbr</code>, burn the LBR, withdraw the corresponding assets from the
LBR reserve, and deposit them to
<code>cap.address</code>.
The
<code>payer</code> address in the
<code> RecievedPaymentEvent</code>s emitted by this function will be the LBR
reserve address to signify that this was a special payment that credits
<code>cap.address</code>'s balance and credits the LBR reserve.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_unstaple_lbr">unstaple_lbr</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_unstaple_lbr">unstaple_lbr</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>, amount_lbr: u64)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // <b>use</b> the reserved address <b>as</b> the payee because the funds will be burned
    <b>let</b> lbr = <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(cap, <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), amount_lbr, x"");
    <b>let</b> (coin1, coin2) = <a href="LBR.md#0x1_LBR_unpack">LBR::unpack</a>(lbr);
    // These funds come from the <a href="LBR.md#0x1_LBR">LBR</a> reserve, so <b>use</b> the <a href="LBR.md#0x1_LBR">LBR</a> reserve address <b>as</b> the payer
    <b>let</b> payer_address = <a href="LBR.md#0x1_LBR_reserve_address">LBR::reserve_address</a>();
    <b>let</b> payee_address = cap.account_address;
    <a href="#0x1_LibraAccount_deposit">deposit</a>(payer_address, payee_address, coin1, x"", x"");
    <a href="#0x1_LibraAccount_deposit">deposit</a>(payer_address, payee_address, coin2, x"", x"")
}
</code></pre>



</details>

<a name="0x1_LibraAccount_deposit"></a>

## Function `deposit`

Record a payment of
<code>to_deposit</code> from
<code>payer</code> to
<code>payee</code> with the attached
<code>metadata</code>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(payer: address, payee: address, to_deposit: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(
    payer: address,
    payee: address,
    to_deposit: <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>assert</b>(!<a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(payee), EACCOUNT_FROZEN);

    // Check that the `to_deposit` coin is non-zero
    <b>let</b> deposit_value = <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&to_deposit);
    <b>assert</b>(deposit_value &gt; 0, ECOIN_DEPOSIT_IS_ZERO);
    // Check that an account exists at `payee`
    <b>assert</b>(exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(payee), EPAYEE_DOES_NOT_EXIST);
    // Check that `payee` can accept payments in `Token`
    <b>assert</b>(exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee), EPAYEE_CANT_ACCEPT_CURRENCY_TYPE);

    // Check that the payment complies with dual attestation rules
    <a href="DualAttestation.md#0x1_DualAttestation_assert_payment_ok">DualAttestation::assert_payment_ok</a>&lt;Token&gt;(
        payer, payee, deposit_value, <b>copy</b> metadata, metadata_signature
    );
    // Ensure that this deposit is compliant with the account limits on
    // this account.
    <b>if</b> (<a href="#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>false</b>)) {
        <b>assert</b>(
            <a href="AccountLimits.md#0x1_AccountLimits_update_deposit_limits">AccountLimits::update_deposit_limits</a>&lt;Token&gt;(
                deposit_value,
                <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(payee),
                &borrow_global&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).limits_cap
            ),
            EDEPOSIT_EXCEEDS_LIMITS
        )
    };

    // Deposit the `to_deposit` coin
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> borrow_global_mut&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee).coin, to_deposit);

    // Log a received event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(payee).received_events,
        <a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a> {
            amount: deposit_value,
            currency_code: <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;Token&gt;(),
            payer,
            metadata
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_tiered_mint"></a>

## Function `tiered_mint`

Mint 'mint_amount' to 'designated_dealer_address' for 'tier_index' tier.
Max valid tier index is 3 since there are max 4 tiers per DD.
Sender should be treasury compliance account and receiver authorized DD.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_tiered_mint">tiered_mint</a>&lt;Token&gt;(tc_account: &signer, designated_dealer_address: address, mint_amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_tiered_mint">tiered_mint</a>&lt;Token&gt;(
    tc_account: &signer,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> coin = <a href="DesignatedDealer.md#0x1_DesignatedDealer_tiered_mint">DesignatedDealer::tiered_mint</a>&lt;Token&gt;(
        tc_account, mint_amount, designated_dealer_address, tier_index
    );
    // Use the reserved address <b>as</b> the payer because the funds did not come from an existing
    // balance
    <a href="#0x1_LibraAccount_deposit">deposit</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), designated_dealer_address, coin, x"", x"")
}
</code></pre>



</details>

<a name="0x1_LibraAccount_cancel_burn"></a>

## Function `cancel_burn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_cancel_burn">cancel_burn</a>&lt;Token&gt;(
    account: &signer,
    preburn_address: address,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> coin = <a href="Libra.md#0x1_Libra_cancel_burn">Libra::cancel_burn</a>&lt;Token&gt;(account, preburn_address);
    // record both sender and recipient <b>as</b> `preburn_address`: the coins are moving from
    // `preburn_address`'s `Preburn` <b>resource</b> <b>to</b> its balance
    <a href="#0x1_LibraAccount_deposit">deposit</a>(preburn_address, preburn_address, coin, x"", x"")
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_from_balance"></a>

## Function `withdraw_from_balance`

Helper to withdraw
<code>amount</code> from the given account balance and return the withdrawn Libra<Token>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(payer: address, payee: address, balance: &<b>mut</b> <a href="#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;, amount: u64): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(
    payer: address,
    payee: address,
    balance: &<b>mut</b> <a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;,
    amount: u64
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt; <b>acquires</b> <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>assert</b>(!<a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(payer), EACCOUNT_FROZEN);
    // Make sure that this withdrawal is compliant with the limits on
    // the account <b>if</b> it's a inter-<a href="VASP.md#0x1_VASP">VASP</a> transfer,
    <b>if</b> (<a href="#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>true</b>)) {
        <b>let</b> can_withdraw = <a href="AccountLimits.md#0x1_AccountLimits_update_withdrawal_limits">AccountLimits::update_withdrawal_limits</a>&lt;Token&gt;(
                amount,
                <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(payer),
                &borrow_global&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).limits_cap
        );
        <b>assert</b>(can_withdraw, EWITHDRAWAL_EXCEEDS_LIMITS);
    };
    <b>let</b> coin = &<b>mut</b> balance.coin;
    // Abort <b>if</b> this withdrawal would make the `payer`'s balance go negative
    <b>assert</b>(<a href="Libra.md#0x1_Libra_value">Libra::value</a>(coin) &gt;= amount, EINSUFFICIENT_BALANCE);
    <a href="Libra.md#0x1_Libra_withdraw">Libra::withdraw</a>(coin, amount)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_from"></a>

## Function `withdraw_from`

Withdraw
<code>amount</code>
<code><a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;</code>'s from the account balance under
<code>cap.account_address</code>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;Token&gt;(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, payee: address, amount: u64, metadata: vector&lt;u8&gt;): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;Token&gt;(
    cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt; <b>acquires</b> <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> payer = cap.account_address;
    <b>assert</b>(exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer), EPAYER_DOESNT_HOLD_CURRENCY);
    <b>let</b> account_balance = borrow_global_mut&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer);
    // Load the payer's account and emit an event <b>to</b> record the withdrawal
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(payer).sent_events,
        <a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a> {
            amount,
            currency_code: <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;Token&gt;(),
            payee,
            metadata
        },
    );
    <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(payer, payee, account_balance, amount)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_preburn"></a>

## Function `preburn`

Withdraw
<code>amount</code>
<code><a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;</code>'s from
<code>cap.address</code> and send them to the
<code>Preburn</code>
resource under
<code>dd</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_preburn">preburn</a>&lt;Token&gt;(dd: &signer, cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_preburn">preburn</a>&lt;Token&gt;(
    dd: &signer, cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>, amount: u64
) <b>acquires</b> <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="#0x1_LibraAccount">LibraAccount</a> {
    <a href="Libra.md#0x1_Libra_preburn_to">Libra::preburn_to</a>&lt;Token&gt;(dd, <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>(cap, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(dd), amount, x""))
}
</code></pre>



</details>

<a name="0x1_LibraAccount_extract_withdraw_capability"></a>

## Function `extract_withdraw_capability`

Return a unique capability granting permission to withdraw from the sender's account balance.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_withdraw_capability">extract_withdraw_capability</a>(
    sender: &signer
): <a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a> <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    // Abort <b>if</b> we already extracted the unique withdraw capability for this account.
    <b>assert</b>(!<a href="#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr), EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED);
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(sender_addr);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_restore_withdraw_capability"></a>

## Function `restore_withdraw_capability`

Return the withdraw capability to the account it originally came from


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.withdrawal_capability, cap)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_pay_from"></a>

## Function `pay_from`

Withdraw
<code>amount</code> Libra<Token> from the address embedded in
<code><a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code> and
deposits it into the
<code>payee</code>'s account balance.
The included
<code>metadata</code> will appear in the
<code><a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a></code> and
<code><a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a></code>.
The
<code>metadata_signature</code> will only be checked if this payment is subject to the dual
attestation protocol


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_pay_from">pay_from</a>&lt;Token&gt;(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_pay_from">pay_from</a>&lt;Token&gt;(
    cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(
        *&cap.account_address,
        payee,
        <a href="#0x1_LibraAccount_withdraw_from">withdraw_from</a>(cap, payee, amount, <b>copy</b> metadata),
        metadata,
        metadata_signature
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_rotate_authentication_key"></a>

## Function `rotate_authentication_key`

Rotate the authentication key for the account under cap.account_address


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(
    cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>,
    new_authentication_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>  {
    <b>let</b> sender_account_resource = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    // Don't allow rotating <b>to</b> clearly invalid key
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_authentication_key) == 32, EMALFORMED_AUTHENTICATION_KEY);
    sender_account_resource.authentication_key = new_authentication_key;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_extract_key_rotation_capability"></a>

## Function `extract_key_rotation_capability`

Return a unique capability granting permission to rotate the sender's authentication key


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Abort <b>if</b> we already extracted the unique key rotation capability for this account.
    <b>assert</b>(!<a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(account_address), EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED);
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(account_address);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_restore_key_rotation_capability"></a>

## Function `restore_key_rotation_capability`

Return the key rotation capability to the account it originally came from


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>)
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.key_rotation_capability, cap)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_add_currencies_for_account"></a>

## Function `add_currencies_for_account`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(new_account: &signer, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(
    new_account: &signer,
    add_all_currencies: bool,
) {
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account);
    <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(new_account);
    <b>if</b> (add_all_currencies) {
        <b>if</b> (!exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(new_account);
        };
        <b>if</b> (!exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(new_account);
        };
        <b>if</b> (!exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(new_account_addr)) {
            <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(new_account);
        };
    };
}
</code></pre>



</details>

<a name="0x1_LibraAccount_make_account"></a>

## Function `make_account`

Creates a new account with account at
<code>new_account_address</code> with a balance of
zero in
<code>Token</code> and authentication key
<code>auth_key_prefix</code> |
<code>fresh_address</code>. If
<code>add_all_currencies</code> is true, 0 balances for all available currencies in the system will
also be added.
Aborts if there is already an account at
<code>new_account_address</code>.
Creating an account at address 0x0 will abort as it is a reserved address for the MoveVM.


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account: signer, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_make_account">make_account</a>(
    new_account: signer,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&new_account);
    // cannot create an account at the reserved address 0x0
    <b>assert</b>(new_account_addr != <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), ECANNOT_CREATE_AT_VM_RESERVED);

    // (1) publish <a href="#0x1_LibraAccount">LibraAccount</a>
    <b>let</b> authentication_key = auth_key_prefix;
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(
        &<b>mut</b> authentication_key, <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(<a href="Signer.md#0x1_Signer_borrow_address">Signer::borrow_address</a>(&new_account))
    );
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&authentication_key) == 32, EMALFORMED_AUTHENTICATION_KEY);
    move_to(
        &new_account,
        <a href="#0x1_LibraAccount">LibraAccount</a> {
            authentication_key,
            withdrawal_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
                <a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a> {
                    account_address: new_account_addr
            }),
            key_rotation_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
                <a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a> {
                    account_address: new_account_addr
            }),
            received_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(&new_account),
            sent_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>&gt;(&new_account),
            sequence_number: 0,
        }
    );
    <a href="AccountFreezing.md#0x1_AccountFreezing_create">AccountFreezing::create</a>(&new_account);
    <a href="#0x1_LibraAccount_destroy_signer">destroy_signer</a>(new_account);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_libra_root_account"></a>

## Function `create_libra_root_account`

Creates the libra root account in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_libra_root_account">create_libra_root_account</a>(new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_libra_root_account">create_libra_root_account</a>(
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>assert</b>(new_account_address == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EINVALID_SINGLETON_ADDRESS);
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="SlidingNonce.md#0x1_SlidingNonce_publish_nonce_resource">SlidingNonce::publish_nonce_resource</a>(&new_account, &new_account);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_treasury_compliance_account"></a>

## Function `create_treasury_compliance_account`

Create a treasury/compliance account at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>(lr_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>(
    lr_account: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(lr_account), ENOT_LIBRA_ROOT);
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="SlidingNonce.md#0x1_SlidingNonce_publish_nonce_resource">SlidingNonce::publish_nonce_resource</a>(lr_account, &new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_designated_dealer"></a>

## Function `create_designated_dealer`

Create a designated dealer account at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>, for non synthetic CoinType.
Creates Preburn resource under account 'new_account_address'


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(creator_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(
    creator_account: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
    add_all_currencies: bool,
) {
    <b>let</b> new_dd_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_dd_account);
    <a href="Roles.md#0x1_Roles_new_designated_dealer_role">Roles::new_designated_dealer_role</a>(creator_account, &new_dd_account);
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_publish_designated_dealer_credential">DesignatedDealer::publish_designated_dealer_credential</a>&lt;CoinType&gt;(&new_dd_account, creator_account, add_all_currencies);
    <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;CoinType&gt;(&new_dd_account, add_all_currencies);
    <a href="DualAttestation.md#0x1_DualAttestation_publish_credential">DualAttestation::publish_credential</a>(
        &new_dd_account, creator_account, human_name, base_url, compliance_public_key
    );
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_dd_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_parent_vasp_account"></a>

## Function `create_parent_vasp_account`

Create an account with the ParentVASP role at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code>.  If
<code>add_all_currencies</code> is true, 0 balances for
all available currencies in the system will also be added.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_parent_vasp_account">create_parent_vasp_account</a>&lt;Token&gt;(creator_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_parent_vasp_account">create_parent_vasp_account</a>&lt;Token&gt;(
    creator_account: &signer,  // libra root
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    base_url: vector&lt;u8&gt;,
    compliance_public_key: vector&lt;u8&gt;,
    add_all_currencies: bool
) {
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Roles.md#0x1_Roles_new_parent_vasp_role">Roles::new_parent_vasp_role</a>(creator_account, &new_account);
    <a href="VASP.md#0x1_VASP_publish_parent_vasp_credential">VASP::publish_parent_vasp_credential</a>(&new_account, creator_account);
    <a href="DualAttestation.md#0x1_DualAttestation_publish_credential">DualAttestation::publish_credential</a>(
        &new_account, creator_account, human_name, base_url, compliance_public_key
    );
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(&new_account, add_all_currencies);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_child_vasp_account"></a>

## Function `create_child_vasp_account`

Create an account with the ChildVASP role at
<code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> |
<code>new_account_address</code> and a 0 balance of type
<code>Token</code>. If
<code>add_all_currencies</code> is true, 0 balances for all avaialable currencies in the system will
also be added. This account will be a child of
<code>creator</code>, which must be a ParentVASP.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_child_vasp_account">create_child_vasp_account</a>&lt;Token&gt;(parent: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_child_vasp_account">create_child_vasp_account</a>&lt;Token&gt;(
    parent: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool,
) {
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Roles.md#0x1_Roles_new_child_vasp_role">Roles::new_child_vasp_role</a>(parent, &new_account);
    <a href="VASP.md#0x1_VASP_publish_child_vasp_credential">VASP::publish_child_vasp_credential</a>(
        parent,
        &new_account,
    );
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(&new_account, add_all_currencies);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_signer"></a>

## Function `create_signer`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_create_signer">create_signer</a>(addr: address): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_LibraAccount_create_signer">create_signer</a>(addr: address): signer;
</code></pre>



</details>

<a name="0x1_LibraAccount_destroy_signer"></a>

## Function `destroy_signer`



<pre><code><b>fun</b> <a href="#0x1_LibraAccount_destroy_signer">destroy_signer</a>(sig: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_LibraAccount_destroy_signer">destroy_signer</a>(sig: signer);
</code></pre>



</details>

<a name="0x1_LibraAccount_balance_for"></a>

## Function `balance_for`

Helper to return the u64 value of the
<code>balance</code> for
<code>account</code>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_balance_for">balance_for</a>&lt;Token&gt;(balance: &<a href="#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_balance_for">balance_for</a>&lt;Token&gt;(balance: &<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;): u64 {
    <a href="Libra.md#0x1_Libra_value">Libra::value</a>&lt;Token&gt;(&balance.coin)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_balance"></a>

## Function `balance`

Return the current balance of the account at
<code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(addr: address): u64 <b>acquires</b> <a href="#0x1_LibraAccount_Balance">Balance</a> {
    <a href="#0x1_LibraAccount_balance_for">balance_for</a>(borrow_global&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_LibraAccount_add_currency"></a>

## Function `add_currency`

Add a balance of
<code>Token</code> type to the sending account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(account: &signer) {
    // aborts <b>if</b> `Token` is not a currency type in the system
    <b>assert</b>(<a href="Libra.md#0x1_Libra_is_currency">Libra::is_currency</a>&lt;Token&gt;(), ENOT_A_CURRENCY);
    // Check that an account with this role is allowed <b>to</b> hold funds
    <b>assert</b>(<a href="Roles.md#0x1_Roles_can_hold_balance">Roles::can_hold_balance</a>(account), EROLE_CANT_STORE_BALANCE);
    // aborts <b>if</b> this account already has a balance in `Token`
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(!exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr), EADD_EXISTING_CURRENCY);

    move_to(account, <a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;{ coin: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;Token&gt;() })
}
</code></pre>



</details>

<a name="0x1_LibraAccount_accepts_currency"></a>

## Function `accepts_currency`

Return whether the account at
<code>addr</code> accepts
<code>Token</code> type coins


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_accepts_currency">accepts_currency</a>&lt;Token&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_accepts_currency">accepts_currency</a>&lt;Token&gt;(addr: address): bool {
    exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_sequence_number_for_account"></a>

## Function `sequence_number_for_account`

Helper to return the sequence number field for given
<code>account</code>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="#0x1_LibraAccount">LibraAccount</a>): u64 {
    account.sequence_number
}
</code></pre>



</details>

<a name="0x1_LibraAccount_sequence_number"></a>

## Function `sequence_number`

Return the current sequence number at
<code>addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_sequence_number">sequence_number</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_sequence_number">sequence_number</a>(addr: address): u64 <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <a href="#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_LibraAccount_authentication_key"></a>

## Function `authentication_key`

Return the authentication key for this account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    *&borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x1_LibraAccount_delegated_key_rotation_capability"></a>

## Function `delegated_key_rotation_capability`

Return true if the account at
<code>addr</code> has delegated its key rotation capability


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_delegated_withdraw_capability"></a>

## Function `delegated_withdraw_capability`

Return true if the account at
<code>addr</code> has delegated its withdraw capability


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
<b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_capability_address"></a>

## Function `withdraw_capability_address`

Return a reference to the address associated with the given withdraw capability


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_LibraAccount_key_rotation_capability_address"></a>

## Function `key_rotation_capability_address`

Return a reference to the address associated with the given key rotation capability


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_LibraAccount_exists_at"></a>

## Function `exists_at`

Checks if an account exists at
<code>check_addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_exists_at">exists_at</a>(check_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_exists_at">exists_at</a>(check_addr: address): bool {
    exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(check_addr)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_prologue"></a>

## Function `prologue`

The prologue is invoked at the beginning of every transaction
It verifies:
- The account's auth key matches the transaction's public key
- That the account has enough balance to pay for all of the gas
- That the sequence number matches the transaction's sequence key


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_prologue">prologue</a>&lt;Token&gt;(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_prologue">prologue</a>&lt;Token&gt;(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
    chain_id: u8
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a> {
    <b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);

    // Check that the chain ID stored on-chain matches the chain ID specified by the transaction
    <b>assert</b>(<a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() == chain_id, EPROLOGUE_BAD_CHAIN_ID);

    // Verify that the transaction sender's account exists
    <b>assert</b>(<a href="#0x1_LibraAccount_exists_at">exists_at</a>(transaction_sender), EPROLOGUE_ACCOUNT_DNE);

    // We check whether this account is frozen, <b>if</b> it is no transaction can be sent from it.
    <b>assert</b>(!<a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(transaction_sender), EPROLOGUE_ACCOUNT_FROZEN);

    // Load the transaction sender's account
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(transaction_sender);

    // Check that the hash of the transaction's <b>public</b> key matches the account's auth key
    <b>assert</b>(
        <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) == *&sender_account.authentication_key,
        EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY
    );

    // Check that the account has enough balance for all of the gas
    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
    // Don't grab the balance <b>if</b> the transaction fee is zero
    <b>if</b> (max_transaction_fee &gt; 0) {
        <b>let</b> balance_amount = <a href="#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(transaction_sender);
        <b>assert</b>(balance_amount &gt;= max_transaction_fee, EPROLOGUE_CANT_PAY_GAS_DEPOSIT);
    };

    // Check that the transaction sequence number matches the sequence number of the account
    <b>assert</b>(txn_sequence_number &gt;= sender_account.sequence_number, EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD);
    <b>assert</b>(txn_sequence_number == sender_account.sequence_number, EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW);
    <b>assert</b>(<a href="LibraTransactionTimeout.md#0x1_LibraTransactionTimeout_is_valid_transaction_timestamp">LibraTransactionTimeout::is_valid_transaction_timestamp</a>(txn_expiration_time), EPROLOGUE_TRANSACTION_EXPIRED);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_epilogue"></a>

## Function `epilogue`

Collects gas and bumps the sequence number for executing a transaction


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(sender: address, transaction_fee_amount: u64, txn_sequence_number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(
    sender: address,
    transaction_fee_amount: u64,
    txn_sequence_number: u64,
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // Load the transaction sender's account and balance resources
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(sender);

    // Bump the sequence number
    sender_account.sequence_number = txn_sequence_number + 1;

    <b>if</b> (transaction_fee_amount &gt; 0) {
        <b>let</b> sender_balance = borrow_global_mut&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(sender);
        <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">TransactionFee::pay_fee</a>(
            <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>(
                sender,
                <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(),
                sender_balance,
                transaction_fee_amount
            )
        )
    }
}
</code></pre>



</details>

<a name="0x1_LibraAccount_success_epilogue"></a>

## Function `success_epilogue`

The success_epilogue is invoked at the end of successfully executed transactions.


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_success_epilogue">success_epilogue</a>&lt;Token&gt;(account: &signer, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_success_epilogue">success_epilogue</a>&lt;Token&gt;(
    account: &signer,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);

    // Charge for gas
    <b>let</b> transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);

    // Load the transaction sender's balance <b>resource</b> only <b>if</b> it exists. If it doesn't we default the value <b>to</b> 0
    <b>let</b> sender_balance = <b>if</b> (exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(sender)) <a href="#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(sender) <b>else</b> 0;
    <b>assert</b>(sender_balance &gt;= transaction_fee_amount, EPROLOGUE_CANT_PAY_GAS_DEPOSIT);
    <a href="#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(sender, transaction_fee_amount, txn_sequence_number);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_failure_epilogue"></a>

## Function `failure_epilogue`

The failure_epilogue is invoked at the end of transactions when the transaction is aborted during execution or
during
<code>success_epilogue</code>.


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_failure_epilogue">failure_epilogue</a>&lt;Token&gt;(account: &signer, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_failure_epilogue">failure_epilogue</a>&lt;Token&gt;(
    account: &signer,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64
) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a>, <a href="#0x1_LibraAccount_Balance">Balance</a>, <a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Charge for gas
    <b>let</b> transaction_fee_amount = txn_gas_price * (txn_max_gas_units - gas_units_remaining);

    <a href="#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(sender, transaction_fee_amount, txn_sequence_number);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_bump_sequence_number"></a>

## Function `bump_sequence_number`

Bump the sequence number of an account. This function should be used only for bumping the sequence number when
a writeset transaction is committed.


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_bump_sequence_number">bump_sequence_number</a>(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_bump_sequence_number">bump_sequence_number</a>(signer: &signer) <b>acquires</b> <a href="#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    sender_account.sequence_number = sender_account.sequence_number + 1;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_validator_account"></a>

## Function `create_validator_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_validator_account">create_validator_account</a>(creator_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_validator_account">create_validator_account</a>(
    creator_account: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
) {
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    // The creator account is verified <b>to</b> have the libra root role in `<a href="Roles.md#0x1_Roles_new_validator_role">Roles::new_validator_role</a>`
    <a href="Roles.md#0x1_Roles_new_validator_role">Roles::new_validator_role</a>(creator_account, &new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="ValidatorConfig.md#0x1_ValidatorConfig_publish">ValidatorConfig::publish</a>(&new_account, creator_account, human_name);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_validator_operator_account"></a>

## Function `create_validator_operator_account`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_validator_operator_account">create_validator_operator_account</a>(creator_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_validator_operator_account">create_validator_operator_account</a>(
    creator_account: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
) {
    <b>let</b> new_account = <a href="#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    // The creator account is verified <b>to</b> have the libra root role in `<a href="Roles.md#0x1_Roles_new_validator_operator_role">Roles::new_validator_operator_role</a>`
    <a href="Roles.md#0x1_Roles_new_validator_operator_role">Roles::new_validator_operator_role</a>(creator_account, &new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_publish">ValidatorOperatorConfig::publish</a>(&new_account, creator_account, human_name);
    <a href="#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_Specification"></a>

## Specification


<a name="0x1_LibraAccount_Specification_should_track_limits_for_account"></a>

### Function `should_track_limits_for_account`


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(payer: address, payee: address, is_withdrawal: bool): bool
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>ensures</b> result == <a href="#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, is_withdrawal);
</code></pre>




<a name="0x1_LibraAccount_spec_has_published_account_limits"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_has_published_account_limits">spec_has_published_account_limits</a>&lt;Token&gt;(addr: address): bool {
    <b>if</b> (<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(addr)) <a href="VASP.md#0x1_VASP_spec_has_account_limits">VASP::spec_has_account_limits</a>&lt;Token&gt;(addr)
    <b>else</b> <a href="AccountLimits.md#0x1_AccountLimits_has_window_published">AccountLimits::has_window_published</a>&lt;Token&gt;(addr)
}
<a name="0x1_LibraAccount_spec_should_track_limits_for_account"></a>
<b>define</b> <a href="#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(
    payer: address, payee: address, is_withdrawal: bool
): bool {
    <b>if</b> (is_withdrawal) {
        <a href="#0x1_LibraAccount_spec_has_published_account_limits">spec_has_published_account_limits</a>&lt;Token&gt;(payer) &&
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payer) &&
        (!<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payee) || !<a href="VASP.md#0x1_VASP_spec_is_same_vasp">VASP::spec_is_same_vasp</a>(payer, payee))
    } <b>else</b> {
        <a href="#0x1_LibraAccount_spec_has_published_account_limits">spec_has_published_account_limits</a>&lt;Token&gt;(payee) &&
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payee) &&
        (!<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payer) || !<a href="VASP.md#0x1_VASP_spec_is_same_vasp">VASP::spec_is_same_vasp</a>(payee, payer))
    }
}
</code></pre>



<a name="0x1_LibraAccount_Specification_deposit"></a>

### Function `deposit`


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(payer: address, payee: address, to_deposit: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_LibraAccount_DepositAbortsIf">DepositAbortsIf</a>&lt;Token&gt;{amount: to_deposit.value};
<b>include</b> <a href="#0x1_LibraAccount_DepositEnsures">DepositEnsures</a>&lt;Token&gt;{amount: to_deposit.value};
</code></pre>




<a name="0x1_LibraAccount_DepositAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraAccount_DepositAbortsIf">DepositAbortsIf</a>&lt;Token&gt; {
    payer: address;
    payee: address;
    amount: u64;
    metadata_signature: vector&lt;u8&gt;;
    metadata: vector&lt;u8&gt;;
    <b>aborts_if</b> <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">AccountFreezing::spec_account_is_frozen</a>(payee);
    <b>aborts_if</b> amount == 0;
    <b>include</b> <a href="DualAttestation.md#0x1_DualAttestation_AssertPaymentOkAbortsIf">DualAttestation::AssertPaymentOkAbortsIf</a>&lt;Token&gt;{value: amount};
    <b>aborts_if</b>
        <a href="#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>false</b>) &&
        !<a href="#0x1_LibraAccount_spec_has_account_operations_cap">spec_has_account_operations_cap</a>();
    <b>include</b>
        <a href="#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>false</b>) ==&gt;
        <a href="AccountLimits.md#0x1_AccountLimits_UpdateDepositLimitsAbortsIf">AccountLimits::UpdateDepositLimitsAbortsIf</a>&lt;Token&gt; {
            addr: <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payee),
        };
    <b>aborts_if</b>
        <a href="#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>false</b>) &&
        !<a href="AccountLimits.md#0x1_AccountLimits_spec_update_deposit_limits">AccountLimits::spec_update_deposit_limits</a>&lt;Token&gt;(amount, <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payee));
    <b>aborts_if</b> !exists&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee);
    <b>aborts_if</b> <b>global</b>&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee).coin.value + amount &gt; max_u64();
    <b>aborts_if</b> !exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(payee);
    <b>include</b> <a href="Libra.md#0x1_Libra_CurrencyCodeAbortsIf">Libra::CurrencyCodeAbortsIf</a>&lt;Token&gt;;
}
</code></pre>




<a name="0x1_LibraAccount_DepositEnsures"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraAccount_DepositEnsures">DepositEnsures</a>&lt;Token&gt; {
    payer: address;
    payee: address;
    amount: u64;
    <b>ensures</b> <b>global</b>&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee).coin.value == <b>old</b>(<b>global</b>&lt;<a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee).coin.value) + amount;
}
</code></pre>



<a name="0x1_LibraAccount_Specification_withdraw_from_balance"></a>

### Function `withdraw_from_balance`


<pre><code><b>fun</b> <a href="#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(payer: address, payee: address, balance: &<b>mut</b> <a href="#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;, amount: u64): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;
</code></pre>




<pre><code><b>include</b> <a href="#0x1_LibraAccount_WithdrawFromBalanceAbortsIf">WithdrawFromBalanceAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="#0x1_LibraAccount_WithdrawFromBalanceEnsures">WithdrawFromBalanceEnsures</a>&lt;Token&gt;;
</code></pre>




<a name="0x1_LibraAccount_WithdrawFromBalanceAbortsIf"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraAccount_WithdrawFromBalanceAbortsIf">WithdrawFromBalanceAbortsIf</a>&lt;Token&gt; {
    payer: address;
    payee: address;
    balance: <a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;;
    amount: u64;
    <b>aborts_if</b> <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">AccountFreezing::spec_account_is_frozen</a>(payer);
    <b>include</b>
        <a href="#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>true</b>) ==&gt;
        <a href="AccountLimits.md#0x1_AccountLimits_UpdateWithdrawalLimitsAbortsIf">AccountLimits::UpdateWithdrawalLimitsAbortsIf</a>&lt;Token&gt; {
            addr: <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payer),
        };
    <b>aborts_if</b>
        <a href="#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>true</b>) &&
        (   !<a href="#0x1_LibraAccount_spec_has_account_operations_cap">spec_has_account_operations_cap</a>() ||
            !<a href="AccountLimits.md#0x1_AccountLimits_spec_update_withdrawal_limits">AccountLimits::spec_update_withdrawal_limits</a>&lt;Token&gt;(amount, <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payer))
        );
    <b>aborts_if</b> balance.coin.value &lt; amount;
}
</code></pre>




<a name="0x1_LibraAccount_WithdrawFromBalanceEnsures"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraAccount_WithdrawFromBalanceEnsures">WithdrawFromBalanceEnsures</a>&lt;Token&gt; {
    balance: <a href="#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;;
    amount: u64;
    result: <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;;
    <b>ensures</b> balance.coin.value == <b>old</b>(balance.coin.value) - amount;
    <b>ensures</b> result.value == amount;
}
</code></pre>



<a name="0x1_LibraAccount_Specification_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(cap: &<a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
<b>aborts_if</b> len(new_authentication_key) != 32;
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address).authentication_key == new_authentication_key;
</code></pre>




<a name="0x1_LibraAccount_spec_rotate_authentication_key"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_rotate_authentication_key">spec_rotate_authentication_key</a>(addr: address, new_authentication_key: vector&lt;u8&gt;): bool {
    <b>global</b>&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).authentication_key == new_authentication_key
}
</code></pre>



<a name="0x1_LibraAccount_Specification_extract_key_rotation_capability"></a>

### Function `extract_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="0x1_LibraAccount_Specification_restore_key_rotation_capability"></a>

### Function `restore_key_rotation_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
<b>aborts_if</b> !<a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(cap.account_address);
<b>ensures</b> <a href="#0x1_LibraAccount_spec_holds_own_key_rotation_cap">spec_holds_own_key_rotation_cap</a>(cap.account_address);
</code></pre>



<a name="0x1_LibraAccount_Specification_create_designated_dealer"></a>

### Function `create_designated_dealer`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(creator_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, base_url: vector&lt;u8&gt;, compliance_public_key: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>




<pre><code>pragma verify;
</code></pre>


Returns field
<code>key_rotation_capability</code> of the
LibraAccount under
<code>addr</code>.


<a name="0x1_LibraAccount_spec_get_key_rotation_cap"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_get_key_rotation_cap">spec_get_key_rotation_cap</a>(addr: address): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>&gt; {
    <b>global</b>&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).key_rotation_capability
}
</code></pre>


Returns true if the LibraAccount at
<code>addr</code> holds
<code><a href="#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a></code> for itself.


<a name="0x1_LibraAccount_spec_holds_own_key_rotation_cap"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_holds_own_key_rotation_cap">spec_holds_own_key_rotation_cap</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<a href="#0x1_LibraAccount_spec_get_key_rotation_cap">spec_get_key_rotation_cap</a>(addr))
    && addr == <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(
        <a href="#0x1_LibraAccount_spec_get_key_rotation_cap">spec_get_key_rotation_cap</a>(addr)).account_address
}
</code></pre>


Returns true if
<code><a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a></code> is published.


<a name="0x1_LibraAccount_spec_has_account_operations_cap"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_has_account_operations_cap">spec_has_account_operations_cap</a>(): bool {
    exists&lt;<a href="#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>())
}
<a name="0x1_LibraAccount_spec_has_key_rotation_cap"></a>
<b>define</b> <a href="#0x1_LibraAccount_spec_has_key_rotation_cap">spec_has_key_rotation_cap</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).key_rotation_capability)
}
</code></pre>


Returns field
<code>withdrawal_capability</code> of LibraAccount under
<code>addr</code>.


<a name="0x1_LibraAccount_spec_get_withdraw_cap"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_get_withdraw_cap">spec_get_withdraw_cap</a>(addr: address): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>&gt; {
    <b>global</b>&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr).withdrawal_capability
}
</code></pre>


Returns true if the LibraAccount at
<code>addr</code> holds a
<code><a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code>.


<a name="0x1_LibraAccount_spec_has_withdraw_cap"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_has_withdraw_cap">spec_has_withdraw_cap</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<a href="#0x1_LibraAccount_spec_get_withdraw_cap">spec_get_withdraw_cap</a>(addr))
}
</code></pre>


Returns true if the LibraAccount at
<code>addr</code> holds
<code><a href="#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code> for itself.


<a name="0x1_LibraAccount_spec_holds_own_withdraw_cap"></a>


<pre><code><b>define</b> <a href="#0x1_LibraAccount_spec_holds_own_withdraw_cap">spec_holds_own_withdraw_cap</a>(addr: address): bool {
    <a href="#0x1_LibraAccount_spec_has_withdraw_cap">spec_has_withdraw_cap</a>(addr)
    && addr == <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(<a href="#0x1_LibraAccount_spec_get_withdraw_cap">spec_get_withdraw_cap</a>(addr)).account_address
}
</code></pre>




<a name="0x1_LibraAccount_EnsuresHasKeyRotationCap"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraAccount_EnsuresHasKeyRotationCap">EnsuresHasKeyRotationCap</a> {
    account: signer;
    <b>ensures</b> <a href="#0x1_LibraAccount_spec_has_key_rotation_cap">spec_has_key_rotation_cap</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
}
</code></pre>




<a name="0x1_LibraAccount_EnsuresWithdrawalCap"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraAccount_EnsuresWithdrawalCap">EnsuresWithdrawalCap</a> {
    account: signer;
    <b>ensures</b> <a href="#0x1_LibraAccount_spec_has_withdraw_cap">spec_has_withdraw_cap</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
}
</code></pre>



the permission "RotateAuthenticationKey(addr)" is granted to the account at addr [B27].


<pre><code><b>apply</b> <a href="#0x1_LibraAccount_EnsuresHasKeyRotationCap">EnsuresHasKeyRotationCap</a>{account: new_account} <b>to</b> make_account;
</code></pre>


the permission "WithdrawalCapability(addr)" is granted to the account at addr [B28].


<pre><code><b>apply</b> <a href="#0x1_LibraAccount_EnsuresWithdrawalCap">EnsuresWithdrawalCap</a>{account: new_account} <b>to</b> make_account;
</code></pre>



The LibraAccount under addr holds either no withdraw capability
(withdraw cap has been delegated) or the withdraw capability for addr itself.


<pre><code><b>invariant</b> [<b>global</b>] forall addr1: address where exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr1):
    <a href="#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr1) || <a href="#0x1_LibraAccount_spec_holds_own_withdraw_cap">spec_holds_own_withdraw_cap</a>(addr1);
</code></pre>


The LibraAccount under addr holds either no key rotation capability
(key rotation cap has been delegated) or the key rotation capability for addr itself.


<pre><code><b>invariant</b> [<b>global</b>] forall addr1: address where exists&lt;<a href="#0x1_LibraAccount">LibraAccount</a>&gt;(addr1):
    <a href="#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr1) || <a href="#0x1_LibraAccount_spec_holds_own_key_rotation_cap">spec_holds_own_key_rotation_cap</a>(addr1);
</code></pre>


<a name="0x1_LibraAccount"></a>

# Module `0x1::LibraAccount`

The <code><a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a></code> module manages accounts. It defines the <code><a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a></code> resource and
numerous auxilliary data structures. It also defines the prolog and epilog that run
before and after every transaction.


-  [Resource <code><a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a></code>](#0x1_LibraAccount_LibraAccount)
-  [Resource <code><a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a></code>](#0x1_LibraAccount_Balance)
-  [Resource <code><a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code>](#0x1_LibraAccount_WithdrawCapability)
-  [Resource <code><a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a></code>](#0x1_LibraAccount_KeyRotationCapability)
-  [Resource <code><a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a></code>](#0x1_LibraAccount_AccountOperationsCapability)
-  [Resource <code><a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a></code>](#0x1_LibraAccount_LibraWriteSetManager)
-  [Struct <code><a href="LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a></code>](#0x1_LibraAccount_SentPaymentEvent)
-  [Struct <code><a href="LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a></code>](#0x1_LibraAccount_ReceivedPaymentEvent)
-  [Struct <code><a href="LibraAccount.md#0x1_LibraAccount_UpgradeEvent">UpgradeEvent</a></code>](#0x1_LibraAccount_UpgradeEvent)
-  [Struct <code><a href="LibraAccount.md#0x1_LibraAccount_CreateAccountEvent">CreateAccountEvent</a></code>](#0x1_LibraAccount_CreateAccountEvent)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_MAX_U64">MAX_U64</a></code>](#0x1_LibraAccount_MAX_U64)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a></code>](#0x1_LibraAccount_EACCOUNT)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_ESEQUENCE_NUMBER">ESEQUENCE_NUMBER</a></code>](#0x1_LibraAccount_ESEQUENCE_NUMBER)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO">ECOIN_DEPOSIT_IS_ZERO</a></code>](#0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">EDEPOSIT_EXCEEDS_LIMITS</a></code>](#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EROLE_CANT_STORE_BALANCE">EROLE_CANT_STORE_BALANCE</a></code>](#0x1_LibraAccount_EROLE_CANT_STORE_BALANCE)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a></code>](#0x1_LibraAccount_EINSUFFICIENT_BALANCE)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">EWITHDRAWAL_EXCEEDS_LIMITS</a></code>](#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a></code>](#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a></code>](#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code>](#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_ECANNOT_CREATE_AT_VM_RESERVED">ECANNOT_CREATE_AT_VM_RESERVED</a></code>](#0x1_LibraAccount_ECANNOT_CREATE_AT_VM_RESERVED)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED">EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED</a></code>](#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EADD_EXISTING_CURRENCY">EADD_EXISTING_CURRENCY</a></code>](#0x1_LibraAccount_EADD_EXISTING_CURRENCY)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EPAYEE_DOES_NOT_EXIST">EPAYEE_DOES_NOT_EXIST</a></code>](#0x1_LibraAccount_EPAYEE_DOES_NOT_EXIST)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a></code>](#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">EPAYER_DOESNT_HOLD_CURRENCY</a></code>](#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EGAS">EGAS</a></code>](#0x1_LibraAccount_EGAS)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT_OPERATIONS_CAPABILITY">EACCOUNT_OPERATIONS_CAPABILITY</a></code>](#0x1_LibraAccount_EACCOUNT_OPERATIONS_CAPABILITY)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_EWRITESET_MANAGER">EWRITESET_MANAGER</a></code>](#0x1_LibraAccount_EWRITESET_MANAGER)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EACCOUNT_FROZEN">PROLOGUE_EACCOUNT_FROZEN</a></code>](#0x1_LibraAccount_PROLOGUE_EACCOUNT_FROZEN)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY">PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY</a></code>](#0x1_LibraAccount_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD">PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD</a></code>](#0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW">PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW</a></code>](#0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EACCOUNT_DNE">PROLOGUE_EACCOUNT_DNE</a></code>](#0x1_LibraAccount_PROLOGUE_EACCOUNT_DNE)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ECANT_PAY_GAS_DEPOSIT">PROLOGUE_ECANT_PAY_GAS_DEPOSIT</a></code>](#0x1_LibraAccount_PROLOGUE_ECANT_PAY_GAS_DEPOSIT)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ETRANSACTION_EXPIRED">PROLOGUE_ETRANSACTION_EXPIRED</a></code>](#0x1_LibraAccount_PROLOGUE_ETRANSACTION_EXPIRED)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EBAD_CHAIN_ID">PROLOGUE_EBAD_CHAIN_ID</a></code>](#0x1_LibraAccount_PROLOGUE_EBAD_CHAIN_ID)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESCRIPT_NOT_ALLOWED">PROLOGUE_ESCRIPT_NOT_ALLOWED</a></code>](#0x1_LibraAccount_PROLOGUE_ESCRIPT_NOT_ALLOWED)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EMODULE_NOT_ALLOWED">PROLOGUE_EMODULE_NOT_ALLOWED</a></code>](#0x1_LibraAccount_PROLOGUE_EMODULE_NOT_ALLOWED)
-  [Const <code><a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_INVALID_WRITESET_SENDER">PROLOGUE_INVALID_WRITESET_SENDER</a></code>](#0x1_LibraAccount_PROLOGUE_INVALID_WRITESET_SENDER)
-  [Function <code>initialize</code>](#0x1_LibraAccount_initialize)
-  [Function <code>has_published_account_limits</code>](#0x1_LibraAccount_has_published_account_limits)
-  [Function <code>should_track_limits_for_account</code>](#0x1_LibraAccount_should_track_limits_for_account)
-  [Function <code>staple_lbr</code>](#0x1_LibraAccount_staple_lbr)
-  [Function <code>unstaple_lbr</code>](#0x1_LibraAccount_unstaple_lbr)
-  [Function <code>deposit</code>](#0x1_LibraAccount_deposit)
-  [Function <code>tiered_mint</code>](#0x1_LibraAccount_tiered_mint)
-  [Function <code>cancel_burn</code>](#0x1_LibraAccount_cancel_burn)
-  [Function <code>withdraw_from_balance</code>](#0x1_LibraAccount_withdraw_from_balance)
-  [Function <code>withdraw_from</code>](#0x1_LibraAccount_withdraw_from)
-  [Function <code>preburn</code>](#0x1_LibraAccount_preburn)
-  [Function <code>extract_withdraw_capability</code>](#0x1_LibraAccount_extract_withdraw_capability)
-  [Function <code>restore_withdraw_capability</code>](#0x1_LibraAccount_restore_withdraw_capability)
-  [Function <code>pay_from</code>](#0x1_LibraAccount_pay_from)
-  [Function <code>rotate_authentication_key</code>](#0x1_LibraAccount_rotate_authentication_key)
-  [Function <code>extract_key_rotation_capability</code>](#0x1_LibraAccount_extract_key_rotation_capability)
-  [Function <code>restore_key_rotation_capability</code>](#0x1_LibraAccount_restore_key_rotation_capability)
-  [Function <code>add_currencies_for_account</code>](#0x1_LibraAccount_add_currencies_for_account)
-  [Function <code>make_account</code>](#0x1_LibraAccount_make_account)
-  [Function <code>create_libra_root_account</code>](#0x1_LibraAccount_create_libra_root_account)
-  [Function <code>create_treasury_compliance_account</code>](#0x1_LibraAccount_create_treasury_compliance_account)
-  [Function <code>create_designated_dealer</code>](#0x1_LibraAccount_create_designated_dealer)
-  [Function <code>create_parent_vasp_account</code>](#0x1_LibraAccount_create_parent_vasp_account)
-  [Function <code>create_child_vasp_account</code>](#0x1_LibraAccount_create_child_vasp_account)
-  [Function <code>create_signer</code>](#0x1_LibraAccount_create_signer)
-  [Function <code>destroy_signer</code>](#0x1_LibraAccount_destroy_signer)
-  [Function <code>balance_for</code>](#0x1_LibraAccount_balance_for)
-  [Function <code>balance</code>](#0x1_LibraAccount_balance)
-  [Function <code>add_currency</code>](#0x1_LibraAccount_add_currency)
-  [Function <code>accepts_currency</code>](#0x1_LibraAccount_accepts_currency)
-  [Function <code>sequence_number_for_account</code>](#0x1_LibraAccount_sequence_number_for_account)
-  [Function <code>sequence_number</code>](#0x1_LibraAccount_sequence_number)
-  [Function <code>authentication_key</code>](#0x1_LibraAccount_authentication_key)
-  [Function <code>delegated_key_rotation_capability</code>](#0x1_LibraAccount_delegated_key_rotation_capability)
-  [Function <code>delegated_withdraw_capability</code>](#0x1_LibraAccount_delegated_withdraw_capability)
-  [Function <code>withdraw_capability_address</code>](#0x1_LibraAccount_withdraw_capability_address)
-  [Function <code>key_rotation_capability_address</code>](#0x1_LibraAccount_key_rotation_capability_address)
-  [Function <code>exists_at</code>](#0x1_LibraAccount_exists_at)
-  [Function <code>module_prologue</code>](#0x1_LibraAccount_module_prologue)
-  [Function <code>script_prologue</code>](#0x1_LibraAccount_script_prologue)
-  [Function <code>writeset_prologue</code>](#0x1_LibraAccount_writeset_prologue)
-  [Function <code>prologue_common</code>](#0x1_LibraAccount_prologue_common)
-  [Function <code>epilogue</code>](#0x1_LibraAccount_epilogue)
-  [Function <code>writeset_epilogue</code>](#0x1_LibraAccount_writeset_epilogue)
-  [Function <code>create_validator_account</code>](#0x1_LibraAccount_create_validator_account)
-  [Function <code>create_validator_operator_account</code>](#0x1_LibraAccount_create_validator_operator_account)
-  [Module Specification](#@Module_Specification_0)


<a name="0x1_LibraAccount_LibraAccount"></a>

## Resource `LibraAccount`

An <code>address</code> is a Libra Account iff it has a published LibraAccount resource.


<pre><code><b>resource</b> <b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>authentication_key: vector&lt;u8&gt;</code>
</dt>
<dd>
 The current authentication key.
 This can be different from the key used to create the account
</dd>
<dt>
<code>withdrawal_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>&gt;</code>
</dt>
<dd>
 A <code>withdrawal_capability</code> allows whoever holds this capability
 to withdraw from the account. At the time of account creation
 this capability is stored in this option. It can later be
 and can also be restored via <code>restore_withdraw_capability</code>.
</dd>
<dt>
<code>key_rotation_capability: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>&gt;</code>
</dt>
<dd>
 A <code>key_rotation_capability</code> allows whoever holds this capability
 the ability to rotate the authentication key for the account. At
 the time of account creation this capability is stored in this
 option. It can later be "extracted" from this field via
 <code>extract_key_rotation_capability</code>, and can also be restored via
 <code>restore_key_rotation_capability</code>.
</dd>
<dt>
<code>received_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">LibraAccount::ReceivedPaymentEvent</a>&gt;</code>
</dt>
<dd>
 Event handle to which ReceivePaymentEvents are emitted when
 payments are received.
</dd>
<dt>
<code>sent_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">LibraAccount::SentPaymentEvent</a>&gt;</code>
</dt>
<dd>
 Event handle to which SentPaymentEvents are emitted when
 payments are sent.
</dd>
<dt>
<code>sequence_number: u64</code>
</dt>
<dd>
 The current sequence number of the account.
 Incremented by one each time a transaction is submitted by
 this account.
</dd>
</dl>


</details>

<a name="0x1_LibraAccount_Balance"></a>

## Resource `Balance`

A resource that holds the total value of currency of type <code>Token</code>
currently held by the account.


<pre><code><b>resource</b> <b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>coin: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;</code>
</dt>
<dd>
 Stores the value of the balance in its balance field. A coin has
 a <code>value</code> field. The amount of money in the balance is changed
 by modifying this field.
</dd>
</dl>


</details>

<a name="0x1_LibraAccount_WithdrawCapability"></a>

## Resource `WithdrawCapability`

The holder of WithdrawCapability for account_address can withdraw Libra from
account_address/LibraAccount/balance.
There is at most one WithdrawCapability in existence for a given address.


<pre><code><b>resource</b> <b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: address</code>
</dt>
<dd>
 Address that WithdrawCapability was associated with when it was created.
 This field does not change.
</dd>
</dl>


</details>

<a name="0x1_LibraAccount_KeyRotationCapability"></a>

## Resource `KeyRotationCapability`

The holder of KeyRotationCapability for account_address can rotate the authentication key for
account_address (i.e., write to account_address/LibraAccount/authentication_key).
There is at most one KeyRotationCapability in existence for a given address.


<pre><code><b>resource</b> <b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: address</code>
</dt>
<dd>
 Address that KeyRotationCapability was associated with when it was created.
 This field does not change.
</dd>
</dl>


</details>

<a name="0x1_LibraAccount_AccountOperationsCapability"></a>

## Resource `AccountOperationsCapability`

A wrapper around an <code>AccountLimitMutationCapability</code> which is used to check for account limits
and to record freeze/unfreeze events.


<pre><code><b>resource</b> <b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>limits_cap: <a href="AccountLimits.md#0x1_AccountLimits_AccountLimitMutationCapability">AccountLimits::AccountLimitMutationCapability</a></code>
</dt>
<dd>

</dd>
<dt>
<code>creation_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_CreateAccountEvent">LibraAccount::CreateAccountEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_LibraWriteSetManager"></a>

## Resource `LibraWriteSetManager`

A resource that holds the event handle for all the past WriteSet transactions that have been committed on chain.


<pre><code><b>resource</b> <b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>upgrade_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_UpgradeEvent">LibraAccount::UpgradeEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraAccount_SentPaymentEvent"></a>

## Struct `SentPaymentEvent`

Message for sent events


<pre><code><b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>
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


<pre><code><b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>
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

<a name="0x1_LibraAccount_UpgradeEvent"></a>

## Struct `UpgradeEvent`

Message for committed WriteSet transaction.


<pre><code><b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_UpgradeEvent">UpgradeEvent</a>
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

<a name="0x1_LibraAccount_CreateAccountEvent"></a>

## Struct `CreateAccountEvent`

Message for creation of a new account


<pre><code><b>struct</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateAccountEvent">CreateAccountEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>created: address</code>
</dt>
<dd>
 Address of the created account
</dd>
<dt>
<code>role_id: u64</code>
</dt>
<dd>
 Role of the created account
</dd>
</dl>


</details>

<a name="0x1_LibraAccount_MAX_U64"></a>

## Const `MAX_U64`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_LibraAccount_EACCOUNT"></a>

## Const `EACCOUNT`

The <code><a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a></code> resource is not in the required state


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>: u64 = 0;
</code></pre>



<a name="0x1_LibraAccount_ESEQUENCE_NUMBER"></a>

## Const `ESEQUENCE_NUMBER`

The account's sequence number has exceeded the maximum representable value


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_ESEQUENCE_NUMBER">ESEQUENCE_NUMBER</a>: u64 = 1;
</code></pre>



<a name="0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO"></a>

## Const `ECOIN_DEPOSIT_IS_ZERO`

Tried to deposit a coin whose value was zero


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO">ECOIN_DEPOSIT_IS_ZERO</a>: u64 = 2;
</code></pre>



<a name="0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS"></a>

## Const `EDEPOSIT_EXCEEDS_LIMITS`

Tried to deposit funds that would have surpassed the account's limits


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">EDEPOSIT_EXCEEDS_LIMITS</a>: u64 = 3;
</code></pre>



<a name="0x1_LibraAccount_EROLE_CANT_STORE_BALANCE"></a>

## Const `EROLE_CANT_STORE_BALANCE`

Tried to create a balance for an account whose role does not allow holding balances


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EROLE_CANT_STORE_BALANCE">EROLE_CANT_STORE_BALANCE</a>: u64 = 4;
</code></pre>



<a name="0x1_LibraAccount_EINSUFFICIENT_BALANCE"></a>

## Const `EINSUFFICIENT_BALANCE`

The account does not hold a large enough balance in the specified currency


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>: u64 = 5;
</code></pre>



<a name="0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS"></a>

## Const `EWITHDRAWAL_EXCEEDS_LIMITS`

The withdrawal of funds would have exceeded the the account's limits


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">EWITHDRAWAL_EXCEEDS_LIMITS</a>: u64 = 6;
</code></pre>



<a name="0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED"></a>

## Const `EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED`

The <code><a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code> for this account has already been extracted


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>: u64 = 7;
</code></pre>



<a name="0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY"></a>

## Const `EMALFORMED_AUTHENTICATION_KEY`

The provided authentication had an invalid length


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>: u64 = 8;
</code></pre>



<a name="0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED"></a>

## Const `EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED`

The <code><a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a></code> for this account has already been extracted


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>: u64 = 9;
</code></pre>



<a name="0x1_LibraAccount_ECANNOT_CREATE_AT_VM_RESERVED"></a>

## Const `ECANNOT_CREATE_AT_VM_RESERVED`

An account cannot be created at the reserved VM address of 0x0


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_ECANNOT_CREATE_AT_VM_RESERVED">ECANNOT_CREATE_AT_VM_RESERVED</a>: u64 = 10;
</code></pre>



<a name="0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED"></a>

## Const `EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED`

The <code><a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code> for this account is not extracted


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED">EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED</a>: u64 = 11;
</code></pre>



<a name="0x1_LibraAccount_EADD_EXISTING_CURRENCY"></a>

## Const `EADD_EXISTING_CURRENCY`

Tried to add a balance in a currency that this account already has


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EADD_EXISTING_CURRENCY">EADD_EXISTING_CURRENCY</a>: u64 = 15;
</code></pre>



<a name="0x1_LibraAccount_EPAYEE_DOES_NOT_EXIST"></a>

## Const `EPAYEE_DOES_NOT_EXIST`

Attempted to send funds to an account that does not exist


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EPAYEE_DOES_NOT_EXIST">EPAYEE_DOES_NOT_EXIST</a>: u64 = 17;
</code></pre>



<a name="0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE"></a>

## Const `EPAYEE_CANT_ACCEPT_CURRENCY_TYPE`

Attempted to send funds in a currency that the receiving account does not hold.
e.g., <code><a href="Libra.md#0x1_Libra">Libra</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;</code> to an account that exists, but does not have a <code><a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;</code> resource


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a>: u64 = 18;
</code></pre>



<a name="0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY"></a>

## Const `EPAYER_DOESNT_HOLD_CURRENCY`

Tried to withdraw funds in a currency that the account does hold


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">EPAYER_DOESNT_HOLD_CURRENCY</a>: u64 = 19;
</code></pre>



<a name="0x1_LibraAccount_EGAS"></a>

## Const `EGAS`

An invalid amount of gas units was provided for execution of the transaction


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EGAS">EGAS</a>: u64 = 20;
</code></pre>



<a name="0x1_LibraAccount_EACCOUNT_OPERATIONS_CAPABILITY"></a>

## Const `EACCOUNT_OPERATIONS_CAPABILITY`

The <code><a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a></code> was not in the required state


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT_OPERATIONS_CAPABILITY">EACCOUNT_OPERATIONS_CAPABILITY</a>: u64 = 22;
</code></pre>



<a name="0x1_LibraAccount_EWRITESET_MANAGER"></a>

## Const `EWRITESET_MANAGER`

The <code><a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a></code> was not in the required state


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_EWRITESET_MANAGER">EWRITESET_MANAGER</a>: u64 = 23;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_EACCOUNT_FROZEN"></a>

## Const `PROLOGUE_EACCOUNT_FROZEN`

Prologue errors. These are separated out from the other errors in this
module since they are mapped separately to major VM statuses, and are
important to the semantics of the system. Those codes also need to be
directly used in aborts instead of augmenting them with a category
via the <code><a href="Errors.md#0x1_Errors">Errors</a></code> module.
const PROLOGUE_EACCOUNT_FROZEN: u64 = 1000;


<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EACCOUNT_FROZEN">PROLOGUE_EACCOUNT_FROZEN</a>: u64 = 1000;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY"></a>

## Const `PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY">PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY</a>: u64 = 1001;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD"></a>

## Const `PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD">PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD</a>: u64 = 1002;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW"></a>

## Const `PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW">PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW</a>: u64 = 1003;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_EACCOUNT_DNE"></a>

## Const `PROLOGUE_EACCOUNT_DNE`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EACCOUNT_DNE">PROLOGUE_EACCOUNT_DNE</a>: u64 = 1004;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_ECANT_PAY_GAS_DEPOSIT"></a>

## Const `PROLOGUE_ECANT_PAY_GAS_DEPOSIT`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ECANT_PAY_GAS_DEPOSIT">PROLOGUE_ECANT_PAY_GAS_DEPOSIT</a>: u64 = 1005;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_ETRANSACTION_EXPIRED"></a>

## Const `PROLOGUE_ETRANSACTION_EXPIRED`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ETRANSACTION_EXPIRED">PROLOGUE_ETRANSACTION_EXPIRED</a>: u64 = 1006;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_EBAD_CHAIN_ID"></a>

## Const `PROLOGUE_EBAD_CHAIN_ID`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EBAD_CHAIN_ID">PROLOGUE_EBAD_CHAIN_ID</a>: u64 = 1007;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_ESCRIPT_NOT_ALLOWED"></a>

## Const `PROLOGUE_ESCRIPT_NOT_ALLOWED`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESCRIPT_NOT_ALLOWED">PROLOGUE_ESCRIPT_NOT_ALLOWED</a>: u64 = 1008;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_EMODULE_NOT_ALLOWED"></a>

## Const `PROLOGUE_EMODULE_NOT_ALLOWED`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EMODULE_NOT_ALLOWED">PROLOGUE_EMODULE_NOT_ALLOWED</a>: u64 = 1009;
</code></pre>



<a name="0x1_LibraAccount_PROLOGUE_INVALID_WRITESET_SENDER"></a>

## Const `PROLOGUE_INVALID_WRITESET_SENDER`



<pre><code><b>const</b> <a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_INVALID_WRITESET_SENDER">PROLOGUE_INVALID_WRITESET_SENDER</a>: u64 = 1010;
</code></pre>



<a name="0x1_LibraAccount_initialize"></a>

## Function `initialize`

Initialize this module. This is only callable from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_initialize">initialize</a>(lr_account: &signer, dummy_auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_initialize">initialize</a>(
    lr_account: &signer,
    dummy_auth_key_prefix: vector&lt;u8&gt;,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    // Operational constraint, not a privilege constraint.
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);

    <a href="LibraAccount.md#0x1_LibraAccount_create_libra_root_account">create_libra_root_account</a>(
        <b>copy</b> dummy_auth_key_prefix,
    );
    <a href="LibraAccount.md#0x1_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>(
        lr_account,
        <b>copy</b> dummy_auth_key_prefix,
    );
}
</code></pre>



</details>

<a name="0x1_LibraAccount_has_published_account_limits"></a>

## Function `has_published_account_limits`

Return <code><b>true</b></code> if <code>addr</code> has already published account limits for <code>Token</code>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_has_published_account_limits">has_published_account_limits</a>&lt;Token&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_has_published_account_limits">has_published_account_limits</a>&lt;Token&gt;(addr: address): bool {
    <b>if</b> (<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(addr)) {
        <a href="VASP.md#0x1_VASP_has_account_limits">VASP::has_account_limits</a>&lt;Token&gt;(addr)
    }
    <b>else</b> {
        <a href="AccountLimits.md#0x1_AccountLimits_has_window_published">AccountLimits::has_window_published</a>&lt;Token&gt;(addr)
    }
}
</code></pre>



</details>

<a name="0x1_LibraAccount_should_track_limits_for_account"></a>

## Function `should_track_limits_for_account`

Returns whether we should track and record limits for the <code>payer</code> or <code>payee</code> account.
Depending on the <code>is_withdrawal</code> flag passed in we determine whether the
<code>payer</code> or <code>payee</code> account is being queried. <code><a href="VASP.md#0x1_VASP">VASP</a>-&gt;any</code> and
<code>any-&gt;<a href="VASP.md#0x1_VASP">VASP</a></code> transfers are tracked in the VASP.


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(payer: address, payee: address, is_withdrawal: bool): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(
    payer: address, payee: address, is_withdrawal: bool
): bool {
    <b>if</b> (is_withdrawal) {
        <a href="LibraAccount.md#0x1_LibraAccount_has_published_account_limits">has_published_account_limits</a>&lt;Token&gt;(payer) &&
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payer) &&
        !<a href="VASP.md#0x1_VASP_is_same_vasp">VASP::is_same_vasp</a>(payer, payee)
    } <b>else</b> {
        <a href="LibraAccount.md#0x1_LibraAccount_has_published_account_limits">has_published_account_limits</a>&lt;Token&gt;(payee) &&
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payee) &&
        !<a href="VASP.md#0x1_VASP_is_same_vasp">VASP::is_same_vasp</a>(payee, payer)
    }
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, is_withdrawal);
</code></pre>




<a name="0x1_LibraAccount_spec_has_published_account_limits"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_has_published_account_limits">spec_has_published_account_limits</a>&lt;Token&gt;(addr: address): bool {
    <b>if</b> (<a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(addr)) <a href="VASP.md#0x1_VASP_spec_has_account_limits">VASP::spec_has_account_limits</a>&lt;Token&gt;(addr)
    <b>else</b> <a href="AccountLimits.md#0x1_AccountLimits_has_window_published">AccountLimits::has_window_published</a>&lt;Token&gt;(addr)
}
<a name="0x1_LibraAccount_spec_should_track_limits_for_account"></a>
<b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(
    payer: address, payee: address, is_withdrawal: bool
): bool {
    <b>if</b> (is_withdrawal) {
        <a href="LibraAccount.md#0x1_LibraAccount_spec_has_published_account_limits">spec_has_published_account_limits</a>&lt;Token&gt;(payer) &&
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payer) &&
        !<a href="VASP.md#0x1_VASP_spec_is_same_vasp">VASP::spec_is_same_vasp</a>(payer, payee)
    } <b>else</b> {
        <a href="LibraAccount.md#0x1_LibraAccount_spec_has_published_account_limits">spec_has_published_account_limits</a>&lt;Token&gt;(payee) &&
        <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(payee) &&
        !<a href="VASP.md#0x1_VASP_spec_is_same_vasp">VASP::spec_is_same_vasp</a>(payee, payer)
    }
}
</code></pre>



</details>

<a name="0x1_LibraAccount_staple_lbr"></a>

## Function `staple_lbr`

Use <code>cap</code> to mint <code>amount_lbr</code> LBR by withdrawing the appropriate quantity of reserve assets
from <code>cap.address</code>, giving them to the LBR reserve, and depositing the LBR into
<code>cap.address</code>.
The <code>payee</code> address in the <code><a href="LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a></code>s emitted by this function is the LBR reserve
address to signify that this was a special payment that debits the <code>cap.addr</code>'s balance and
credits the LBR reserve.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_staple_lbr">staple_lbr</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_staple_lbr">staple_lbr</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>, amount_lbr: u64)
<b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>, <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <b>let</b> cap_address = cap.account_address;
    // <b>use</b> the <a href="LBR.md#0x1_LBR">LBR</a> reserve address <b>as</b> `payee_address`
    <b>let</b> payee_address = <a href="LBR.md#0x1_LBR_reserve_address">LBR::reserve_address</a>();
    <b>let</b> (amount_coin1, amount_coin2) = <a href="LBR.md#0x1_LBR_calculate_component_amounts_for_lbr">LBR::calculate_component_amounts_for_lbr</a>(amount_lbr);
    <b>let</b> coin1 = <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(cap, payee_address, amount_coin1, x"");
    <b>let</b> coin2 = <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(cap, payee_address, amount_coin2, x"");
    // Create `amount_lbr` <a href="LBR.md#0x1_LBR">LBR</a>
    <b>let</b> lbr = <a href="LBR.md#0x1_LBR_create">LBR::create</a>(amount_lbr, coin1, coin2);
    // <b>use</b> the reserved address <b>as</b> the payer for the <a href="LBR.md#0x1_LBR">LBR</a> payment because the funds did not come
    // from an existing balance
    <a href="LibraAccount.md#0x1_LibraAccount_deposit">deposit</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), cap_address, lbr, x"", x"");
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
pragma verify_duration_estimate = 100;
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(cap.account_address);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(cap.account_address);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(cap.account_address);
<b>modifies</b> <b>global</b>&lt;<a href="Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
<b>modifies</b> <b>global</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">LBR::Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
<b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address).withdrawal_capability
    == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address).withdrawal_capability);
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_StapleLBRAbortsIf">StapleLBRAbortsIf</a>;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_StapleLBREnsures">StapleLBREnsures</a>;
</code></pre>




<a name="0x1_LibraAccount_StapleLBRAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_StapleLBRAbortsIf">StapleLBRAbortsIf</a> {
    cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>;
    amount_lbr: u64;
    <a name="0x1_LibraAccount_reserve$62"></a>
    <b>let</b> reserve = <b>global</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">LBR::Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a name="0x1_LibraAccount_amount_coin1$63"></a>
    <b>let</b> amount_coin1 = <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">FixedPoint32::spec_multiply_u64</a>(amount_lbr, reserve.coin1.ratio) + 1;
    <a name="0x1_LibraAccount_amount_coin2$64"></a>
    <b>let</b> amount_coin2 = <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">FixedPoint32::spec_multiply_u64</a>(amount_lbr, reserve.coin2.ratio) + 1;
    <b>aborts_if</b> amount_lbr == 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> reserve.coin1.backing.value + amount_coin1 &gt; <a href="LibraAccount.md#0x1_LibraAccount_MAX_U64">MAX_U64</a> <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
    <b>aborts_if</b> reserve.coin2.backing.value + amount_coin2 &gt; <a href="LibraAccount.md#0x1_LibraAccount_MAX_U64">MAX_U64</a> <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>include</b> <a href="Libra.md#0x1_Libra_MintAbortsIf">Libra::MintAbortsIf</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;{value: amount_lbr};
    <b>include</b> <a href="LBR.md#0x1_LBR_CalculateComponentAmountsForLBRAbortsIf">LBR::CalculateComponentAmountsForLBRAbortsIf</a>;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromAbortsIf">WithdrawFromAbortsIf</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;{
        payee: <a href="LBR.md#0x1_LBR_reserve_address">LBR::reserve_address</a>(), amount: amount_coin1};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromAbortsIf">WithdrawFromAbortsIf</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;{
        payee: <a href="LBR.md#0x1_LBR_reserve_address">LBR::reserve_address</a>(), amount: amount_coin2};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIf">DepositAbortsIf</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;{
        payer: <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(),
        payee: cap.account_address,
        amount: amount_lbr,
        metadata: x"",
        metadata_signature: x"",
    };
    <b>aborts_if</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(cap.account_address).coin.value + amount_lbr &gt; max_u64() <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_LibraAccount_StapleLBREnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_StapleLBREnsures">StapleLBREnsures</a> {
    cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>;
    amount_lbr: u64;
    <a name="0x1_LibraAccount_reserve$65"></a>
    <b>let</b> reserve = <b>global</b>&lt;<a href="LBR.md#0x1_LBR_Reserve">LBR::Reserve</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a name="0x1_LibraAccount_amount_coin1$66"></a>
    <b>let</b> amount_coin1 = <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">FixedPoint32::spec_multiply_u64</a>(amount_lbr, reserve.coin1.ratio) + 1;
    <a name="0x1_LibraAccount_amount_coin2$67"></a>
    <b>let</b> amount_coin2 = <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">FixedPoint32::spec_multiply_u64</a>(amount_lbr, reserve.coin2.ratio) + 1;
    <a name="0x1_LibraAccount_total_value_coin1$68"></a>
    <b>let</b> total_value_coin1 = <b>global</b>&lt;<a href="Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).total_value;
    <a name="0x1_LibraAccount_total_value_coin2$69"></a>
    <b>let</b> total_value_coin2 = <b>global</b>&lt;<a href="Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).total_value;
    <a name="0x1_LibraAccount_total_value_lbr$70"></a>
    <b>let</b> total_value_lbr = <b>global</b>&lt;<a href="Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>()).total_value;
    <b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(cap.account_address).coin.value
        == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(cap.account_address).coin.value) - amount_coin1;
    <b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(cap.account_address).coin.value
        == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(cap.account_address).coin.value) - amount_coin2;
    <b>ensures</b> <a href="Libra.md#0x1_Libra_value">Libra::value</a>(reserve.coin1.backing)
        == <b>old</b>(<a href="Libra.md#0x1_Libra_value">Libra::value</a>(reserve.coin1.backing)) + amount_coin1;
    <b>ensures</b> <a href="Libra.md#0x1_Libra_value">Libra::value</a>(reserve.coin2.backing)
        == <b>old</b>(<a href="Libra.md#0x1_Libra_value">Libra::value</a>(reserve.coin2.backing)) + amount_coin2;
    <b>ensures</b> total_value_coin1 == <b>old</b>(total_value_coin1);
    <b>ensures</b> total_value_coin2 == <b>old</b>(total_value_coin2);
    <b>ensures</b> total_value_lbr == <b>old</b>(total_value_lbr) + amount_lbr;
    <b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(cap.account_address).coin.value
        == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(cap.account_address).coin.value) + amount_lbr;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_unstaple_lbr"></a>

## Function `unstaple_lbr`

Use <code>cap</code> to withdraw <code>amount_lbr</code>, burn the LBR, withdraw the corresponding assets from the
LBR reserve, and deposit them to <code>cap.address</code>.
The <code>payer</code> address in the<code> RecievedPaymentEvent</code>s emitted by this function will be the LBR
reserve address to signify that this was a special payment that credits
<code>cap.address</code>'s balance and credits the LBR reserve.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_unstaple_lbr">unstaple_lbr</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount_lbr: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_unstaple_lbr">unstaple_lbr</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>, amount_lbr: u64)
<b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>, <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    // <b>use</b> the reserved address <b>as</b> the payee because the funds will be burned
    <b>let</b> lbr = <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(cap, <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), amount_lbr, x"");
    <b>let</b> (coin1, coin2) = <a href="LBR.md#0x1_LBR_unpack">LBR::unpack</a>(lbr);
    // These funds come from the <a href="LBR.md#0x1_LBR">LBR</a> reserve, so <b>use</b> the <a href="LBR.md#0x1_LBR">LBR</a> reserve address <b>as</b> the payer
    <b>let</b> payer_address = <a href="LBR.md#0x1_LBR_reserve_address">LBR::reserve_address</a>();
    <b>let</b> payee_address = cap.account_address;
    <a href="LibraAccount.md#0x1_LibraAccount_deposit">deposit</a>(payer_address, payee_address, coin1, x"", x"");
    <a href="LibraAccount.md#0x1_LibraAccount_deposit">deposit</a>(payer_address, payee_address, coin2, x"", x"")
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


> TODO: timeout


<pre><code>pragma verify = <b>false</b>;
</code></pre>



</details>

<a name="0x1_LibraAccount_deposit"></a>

## Function `deposit`

Record a payment of <code>to_deposit</code> from <code>payer</code> to <code>payee</code> with the attached <code>metadata</code>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(payer: address, payee: address, to_deposit: <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(
    payer: address,
    payee: address,
    to_deposit: <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>, <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="AccountFreezing.md#0x1_AccountFreezing_assert_not_frozen">AccountFreezing::assert_not_frozen</a>(payee);

    // Check that the `to_deposit` coin is non-zero
    <b>let</b> deposit_value = <a href="Libra.md#0x1_Libra_value">Libra::value</a>(&to_deposit);
    <b>assert</b>(deposit_value &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_ECOIN_DEPOSIT_IS_ZERO">ECOIN_DEPOSIT_IS_ZERO</a>));
    // Check that an account <b>exists</b> at `payee`
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(payee), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EPAYEE_DOES_NOT_EXIST">EPAYEE_DOES_NOT_EXIST</a>));
    // Check that `payee` can accept payments in `Token`
    <b>assert</b>(
        <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_EPAYEE_CANT_ACCEPT_CURRENCY_TYPE">EPAYEE_CANT_ACCEPT_CURRENCY_TYPE</a>)
    );

    // Check that the payment complies <b>with</b> dual attestation rules
    <a href="DualAttestation.md#0x1_DualAttestation_assert_payment_ok">DualAttestation::assert_payment_ok</a>&lt;Token&gt;(
        payer, payee, deposit_value, <b>copy</b> metadata, metadata_signature
    );
    // Ensure that this deposit is compliant <b>with</b> the account limits on
    // this account.
    <b>if</b> (<a href="LibraAccount.md#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>false</b>)) {
        <b>assert</b>(
            <a href="AccountLimits.md#0x1_AccountLimits_update_deposit_limits">AccountLimits::update_deposit_limits</a>&lt;Token&gt;(
                deposit_value,
                <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(payee),
                &borrow_global&lt;<a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).limits_cap
            ),
            <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="LibraAccount.md#0x1_LibraAccount_EDEPOSIT_EXCEEDS_LIMITS">EDEPOSIT_EXCEEDS_LIMITS</a>)
        )
    };

    // Deposit the `to_deposit` coin
    <a href="Libra.md#0x1_Libra_deposit">Libra::deposit</a>(&<b>mut</b> borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee).coin, to_deposit);

    // Log a received event
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payee).received_events,
        <a href="LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a> {
            amount: deposit_value,
            currency_code: <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;Token&gt;(),
            payer,
            metadata
        }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payee);
<b>modifies</b> <b>global</b>&lt;<a href="AccountLimits.md#0x1_AccountLimits_Window">AccountLimits::Window</a>&lt;Token&gt;&gt;(<a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payee));
<b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payee);
<b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee);
<b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payee).withdrawal_capability
    == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payee).withdrawal_capability);
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIf">DepositAbortsIf</a>&lt;Token&gt;{amount: to_deposit.value};
<b>aborts_if</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee).coin.value + to_deposit.value &gt; max_u64() <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositEnsures">DepositEnsures</a>&lt;Token&gt;{amount: to_deposit.value};
</code></pre>




<a name="0x1_LibraAccount_DepositAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIf">DepositAbortsIf</a>&lt;Token&gt; {
    payer: address;
    payee: address;
    amount: u64;
    metadata_signature: vector&lt;u8&gt;;
    metadata: vector&lt;u8&gt;;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIfRestricted">DepositAbortsIfRestricted</a>&lt;Token&gt;;
    <b>include</b> <a href="AccountFreezing.md#0x1_AccountFreezing_AbortsIfFrozen">AccountFreezing::AbortsIfFrozen</a>{account: payee};
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(payee) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
}
</code></pre>




<a name="0x1_LibraAccount_DepositAbortsIfRestricted"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIfRestricted">DepositAbortsIfRestricted</a>&lt;Token&gt; {
    payer: address;
    payee: address;
    amount: u64;
    metadata_signature: vector&lt;u8&gt;;
    metadata: vector&lt;u8&gt;;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>aborts_if</b> amount == 0 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>include</b> <a href="DualAttestation.md#0x1_DualAttestation_AssertPaymentOkAbortsIf">DualAttestation::AssertPaymentOkAbortsIf</a>&lt;Token&gt;{value: amount};
    <b>include</b>
        <a href="LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>false</b>) ==&gt;
        <a href="AccountLimits.md#0x1_AccountLimits_UpdateDepositLimitsAbortsIf">AccountLimits::UpdateDepositLimitsAbortsIf</a>&lt;Token&gt; {
            addr: <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payee),
        };
    <b>aborts_if</b>
        <a href="LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>false</b>) &&
            !<a href="AccountLimits.md#0x1_AccountLimits_spec_update_deposit_limits">AccountLimits::spec_update_deposit_limits</a>&lt;Token&gt;(amount, <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payee))
        <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;Token&gt;;
}
</code></pre>




<a name="0x1_LibraAccount_DepositEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositEnsures">DepositEnsures</a>&lt;Token&gt; {
    payee: address;
    amount: u64;
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(payee) == <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(payee)) + amount;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_tiered_mint"></a>

## Function `tiered_mint`

Mint 'mint_amount' to 'designated_dealer_address' for 'tier_index' tier.
Max valid tier index is 3 since there are max 4 tiers per DD.
Sender should be treasury compliance account and receiver authorized DD.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_tiered_mint">tiered_mint</a>&lt;Token&gt;(tc_account: &signer, designated_dealer_address: address, mint_amount: u64, tier_index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_tiered_mint">tiered_mint</a>&lt;Token&gt;(
    tc_account: &signer,
    designated_dealer_address: address,
    mint_amount: u64,
    tier_index: u64,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>, <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> coin = <a href="DesignatedDealer.md#0x1_DesignatedDealer_tiered_mint">DesignatedDealer::tiered_mint</a>&lt;Token&gt;(
        tc_account, mint_amount, designated_dealer_address, tier_index
    );
    // Use the reserved address <b>as</b> the payer because the funds did not come from an existing
    // balance
    <a href="LibraAccount.md#0x1_LibraAccount_deposit">deposit</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), designated_dealer_address, coin, x"", x"")
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(designated_dealer_address);
<b>modifies</b> <b>global</b>&lt;<a href="Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_TieredMintAbortsIf">TieredMintAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_TieredMintEnsures">TieredMintEnsures</a>&lt;Token&gt;;
</code></pre>




<a name="0x1_LibraAccount_TieredMintAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_TieredMintAbortsIf">TieredMintAbortsIf</a>&lt;Token&gt; {
    tc_account: signer;
    designated_dealer_address: address;
    mint_amount: u64;
    tier_index: u64;
    <b>include</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_TieredMintAbortsIf">DesignatedDealer::TieredMintAbortsIf</a>&lt;Token&gt;{dd_addr: designated_dealer_address, amount: mint_amount};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIf">DepositAbortsIf</a>&lt;Token&gt;{payer: <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(),
        payee: designated_dealer_address, amount: mint_amount, metadata: x"", metadata_signature: x""};
    <b>aborts_if</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(designated_dealer_address).coin.value + mint_amount &gt; max_u64() <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_LibraAccount_TieredMintEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_TieredMintEnsures">TieredMintEnsures</a>&lt;Token&gt; {
    designated_dealer_address: address;
    mint_amount: u64;
    <a name="0x1_LibraAccount_dealer_balance$71"></a>
    <b>let</b> dealer_balance = <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(designated_dealer_address).coin.value;
    <a name="0x1_LibraAccount_currency_info$72"></a>
    <b>let</b> currency_info = <b>global</b>&lt;<a href="Libra.md#0x1_Libra_CurrencyInfo">Libra::CurrencyInfo</a>&lt;Token&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_CURRENCY_INFO_ADDRESS">CoreAddresses::CURRENCY_INFO_ADDRESS</a>());
}
</code></pre>


Total value of the currency increases by <code>amount</code>.


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_TieredMintEnsures">TieredMintEnsures</a>&lt;Token&gt; {
    <b>ensures</b> currency_info == update_field(<b>old</b>(currency_info), total_value, <b>old</b>(currency_info.total_value) + mint_amount);
}
</code></pre>


The balance of designated dealer increases by <code>amount</code>.


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_TieredMintEnsures">TieredMintEnsures</a>&lt;Token&gt; {
    <b>ensures</b> dealer_balance == <b>old</b>(dealer_balance) + mint_amount;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_cancel_burn"></a>

## Function `cancel_burn`



<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_cancel_burn">cancel_burn</a>&lt;Token&gt;(
    account: &signer,
    preburn_address: address,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>, <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> coin = <a href="Libra.md#0x1_Libra_cancel_burn">Libra::cancel_burn</a>&lt;Token&gt;(account, preburn_address);
    // record both sender and recipient <b>as</b> `preburn_address`: the coins are moving from
    // `preburn_address`'s `Preburn` <b>resource</b> <b>to</b> its balance
    <a href="LibraAccount.md#0x1_LibraAccount_deposit">deposit</a>(preburn_address, preburn_address, coin, x"", x"")
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CancelBurnAbortsIf">CancelBurnAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="Libra.md#0x1_Libra_CancelBurnWithCapEnsures">Libra::CancelBurnWithCapEnsures</a>&lt;Token&gt;;
<a name="0x1_LibraAccount_preburn_value_at_addr$81"></a>
<b>let</b> preburn_value_at_addr = <b>global</b>&lt;<a href="Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;&gt;(preburn_address).to_burn.value;
<a name="0x1_LibraAccount_balance_at_addr$82"></a>
<b>let</b> balance_at_addr = <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(preburn_address);
<b>ensures</b> balance_at_addr == <b>old</b>(balance_at_addr) + <b>old</b>(preburn_value_at_addr);
</code></pre>




<a name="0x1_LibraAccount_CancelBurnAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CancelBurnAbortsIf">CancelBurnAbortsIf</a>&lt;Token&gt; {
    account: signer;
    preburn_address: address;
    <a name="0x1_LibraAccount_amount$73"></a>
    <b>let</b> amount = <b>global</b>&lt;<a href="Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;&gt;(preburn_address).to_burn.value;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Libra.md#0x1_Libra_BurnCapability">Libra::BurnCapability</a>&lt;Token&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account))
        <b>with</b> <a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>;
    <b>include</b> <a href="Libra.md#0x1_Libra_CancelBurnWithCapAbortsIf">Libra::CancelBurnWithCapAbortsIf</a>&lt;Token&gt;;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIf">DepositAbortsIf</a>&lt;Token&gt;{
        payer: preburn_address,
        payee: preburn_address,
        amount: amount,
        metadata: x"",
        metadata_signature: x""
    };
    <b>aborts_if</b> <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(preburn_address) + amount &gt; max_u64() <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_from_balance"></a>

## Function `withdraw_from_balance`

Helper to withdraw <code>amount</code> from the given account balance and return the withdrawn Libra<Token>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(payer: address, payee: address, balance: &<b>mut</b> <a href="LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;, amount: u64): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(
    payer: address,
    payee: address,
    balance: &<b>mut</b> <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;,
    amount: u64
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt; <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="AccountFreezing.md#0x1_AccountFreezing_assert_not_frozen">AccountFreezing::assert_not_frozen</a>(payer);
    // Make sure that this withdrawal is compliant <b>with</b> the limits on
    // the account <b>if</b> it's a inter-<a href="VASP.md#0x1_VASP">VASP</a> transfer,
    <b>if</b> (<a href="LibraAccount.md#0x1_LibraAccount_should_track_limits_for_account">should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>true</b>)) {
        <b>let</b> can_withdraw = <a href="AccountLimits.md#0x1_AccountLimits_update_withdrawal_limits">AccountLimits::update_withdrawal_limits</a>&lt;Token&gt;(
                amount,
                <a href="VASP.md#0x1_VASP_parent_address">VASP::parent_address</a>(payer),
                &borrow_global&lt;<a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).limits_cap
        );
        <b>assert</b>(can_withdraw, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_EXCEEDS_LIMITS">EWITHDRAWAL_EXCEEDS_LIMITS</a>));
    };
    <b>let</b> coin = &<b>mut</b> balance.coin;
    // Abort <b>if</b> this withdrawal would make the `payer`'s balance go negative
    <b>assert</b>(<a href="Libra.md#0x1_Libra_value">Libra::value</a>(coin) &gt;= amount, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="LibraAccount.md#0x1_LibraAccount_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>));
    <a href="Libra.md#0x1_Libra_withdraw">Libra::withdraw</a>(coin, amount)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>modifies</b> <b>global</b>&lt;<a href="AccountLimits.md#0x1_AccountLimits_Window">AccountLimits::Window</a>&lt;Token&gt;&gt;(<a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payer));
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceAbortsIf">WithdrawFromBalanceAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceEnsures">WithdrawFromBalanceEnsures</a>&lt;Token&gt;;
</code></pre>




<a name="0x1_LibraAccount_WithdrawFromBalanceAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceAbortsIf">WithdrawFromBalanceAbortsIf</a>&lt;Token&gt; {
    payer: address;
    payee: address;
    balance: <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;;
    amount: u64;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceNoLimitsAbortsIf">WithdrawFromBalanceNoLimitsAbortsIf</a>&lt;Token&gt;;
    <b>include</b>
        <a href="LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>true</b>) ==&gt;
        <a href="AccountLimits.md#0x1_AccountLimits_UpdateWithdrawalLimitsAbortsIf">AccountLimits::UpdateWithdrawalLimitsAbortsIf</a>&lt;Token&gt; {
            addr: <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payer),
        };
    <b>aborts_if</b>
        <a href="LibraAccount.md#0x1_LibraAccount_spec_should_track_limits_for_account">spec_should_track_limits_for_account</a>&lt;Token&gt;(payer, payee, <b>true</b>) &&
        (   !<a href="LibraAccount.md#0x1_LibraAccount_spec_has_account_operations_cap">spec_has_account_operations_cap</a>() ||
            !<a href="AccountLimits.md#0x1_AccountLimits_spec_update_withdrawal_limits">AccountLimits::spec_update_withdrawal_limits</a>&lt;Token&gt;(amount, <a href="VASP.md#0x1_VASP_spec_parent_address">VASP::spec_parent_address</a>(payer))
        )
        <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_LibraAccount_WithdrawFromBalanceNoLimitsAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceNoLimitsAbortsIf">WithdrawFromBalanceNoLimitsAbortsIf</a>&lt;Token&gt; {
    payer: address;
    payee: address;
    balance: <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;;
    amount: u64;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>include</b> <a href="AccountFreezing.md#0x1_AccountFreezing_AbortsIfFrozen">AccountFreezing::AbortsIfFrozen</a>{account: payer};
    <b>aborts_if</b> balance.coin.value &lt; amount <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
}
</code></pre>




<a name="0x1_LibraAccount_WithdrawFromBalanceEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceEnsures">WithdrawFromBalanceEnsures</a>&lt;Token&gt; {
    balance: <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;;
    amount: u64;
    result: <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;;
    <b>ensures</b> balance.coin.value == <b>old</b>(balance.coin.value) - amount;
    <b>ensures</b> result.value == amount;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_from"></a>

## Function `withdraw_from`

Withdraw <code>amount</code> <code><a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;</code>'s from the account balance under
<code>cap.account_address</code>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;Token&gt;(cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, payee: address, amount: u64, metadata: vector&lt;u8&gt;): <a href="Libra.md#0x1_Libra_Libra">Libra::Libra</a>&lt;Token&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">withdraw_from</a>&lt;Token&gt;(
    cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
): <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt; <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>, <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <b>let</b> payer = cap.account_address;
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(payer), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <b>assert</b>(<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">EPAYER_DOESNT_HOLD_CURRENCY</a>));
    <b>let</b> account_balance = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer);
    // Load the payer's account and emit an event <b>to</b> record the withdrawal
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>&gt;(
        &<b>mut</b> borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer).sent_events,
        <a href="LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a> {
            amount,
            currency_code: <a href="Libra.md#0x1_Libra_currency_code">Libra::currency_code</a>&lt;Token&gt;(),
            payee,
            metadata
        },
    );
    <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from_balance">withdraw_from_balance</a>&lt;Token&gt;(payer, payee, account_balance, amount)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Can only withdraw from the balances of cap.account_address [H17].


<pre><code><b>ensures</b> <b>forall</b> addr1: address <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr1)) && addr1 != cap.account_address:
    <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr1).coin.value == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr1).coin.value);
</code></pre>




<a name="0x1_LibraAccount_payer$83"></a>


<pre><code><b>let</b> payer = cap.account_address;
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer);
<b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer);
<b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer).withdrawal_capability
            == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer).withdrawal_capability);
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromAbortsIf">WithdrawFromAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceEnsures">WithdrawFromBalanceEnsures</a>&lt;Token&gt;{balance: <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer)};
</code></pre>




<a name="0x1_LibraAccount_WithdrawFromAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromAbortsIf">WithdrawFromAbortsIf</a>&lt;Token&gt; {
    cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>;
    payee: address;
    amount: u64;
    <a name="0x1_LibraAccount_payer$59"></a>
    <b>let</b> payer = cap.account_address;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;Token&gt;;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceAbortsIf">WithdrawFromBalanceAbortsIf</a>&lt;Token&gt;{payer: payer, balance: <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer)};
    <b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(payer) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_preburn"></a>

## Function `preburn`

Withdraw <code>amount</code> <code><a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt;</code>'s from <code>cap.address</code> and send them to the <code>Preburn</code>
resource under <code>dd</code>.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_preburn">preburn</a>&lt;Token&gt;(dd: &signer, cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_preburn">preburn</a>&lt;Token&gt;(
    dd: &signer,
    cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>,
    amount: u64
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>, <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>, <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Libra.md#0x1_Libra_preburn_to">Libra::preburn_to</a>&lt;Token&gt;(dd, <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">withdraw_from</a>(cap, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(dd), amount, x""))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<a name="0x1_LibraAccount_dd_addr$84"></a>
<b>let</b> dd_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(dd);
<a name="0x1_LibraAccount_payer$85"></a>
<b>let</b> payer = cap.account_address;
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer);
<b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer);
<b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer).withdrawal_capability
        == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer).withdrawal_capability);
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_PreburnAbortsIf">PreburnAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_PreburnEnsures">PreburnEnsures</a>&lt;Token&gt;{dd_addr: dd_addr, payer: payer};
</code></pre>




<a name="0x1_LibraAccount_PreburnAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_PreburnAbortsIf">PreburnAbortsIf</a>&lt;Token&gt; {
    dd: signer;
    cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>;
    amount: u64;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>{};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromAbortsIf">WithdrawFromAbortsIf</a>&lt;Token&gt;{payee: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(dd)};
    <b>include</b> <a href="Libra.md#0x1_Libra_PreburnToAbortsIf">Libra::PreburnToAbortsIf</a>&lt;Token&gt;{account: dd};
}
</code></pre>




<a name="0x1_LibraAccount_PreburnEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_PreburnEnsures">PreburnEnsures</a>&lt;Token&gt; {
    dd_addr: address;
    payer: address;
    <a name="0x1_LibraAccount_payer_balance$60"></a>
    <b>let</b> payer_balance = <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer).coin.value;
    <a name="0x1_LibraAccount_preburn$61"></a>
    <b>let</b> preburn = <b>global</b>&lt;<a href="Libra.md#0x1_Libra_Preburn">Libra::Preburn</a>&lt;Token&gt;&gt;(dd_addr);
}
</code></pre>


The balance of payer decreases by <code>amount</code>.


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_PreburnEnsures">PreburnEnsures</a>&lt;Token&gt; {
    <b>ensures</b> payer_balance == <b>old</b>(payer_balance) - amount;
}
</code></pre>


The value of preburn at <code>dd_addr</code> increases by <code>amount</code>;


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_PreburnEnsures">PreburnEnsures</a>&lt;Token&gt; {
    <b>include</b> <a href="Libra.md#0x1_Libra_PreburnEnsures">Libra::PreburnEnsures</a>&lt;Token&gt;{preburn: preburn};
}
</code></pre>



</details>

<a name="0x1_LibraAccount_extract_withdraw_capability"></a>

## Function `extract_withdraw_capability`

Return a unique capability granting permission to withdraw from the sender's account balance.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">extract_withdraw_capability</a>(sender: &signer): <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">extract_withdraw_capability</a>(
    sender: &signer
): <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a> <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    // Abort <b>if</b> we already extracted the unique withdraw capability for this account.
    <b>assert</b>(
        !<a href="LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED">EWITHDRAWAL_CAPABILITY_ALREADY_EXTRACTED</a>)
    );
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(sender_addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <b>let</b> account = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(sender_addr);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.withdrawal_capability)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<a name="0x1_LibraAccount_sender_addr$86"></a>
<b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(sender_addr);
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">ExtractWithdrawCapAbortsIf</a>{sender_addr};
<b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(sender_addr);
<b>ensures</b> result == <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">spec_get_withdraw_cap</a>(sender_addr));
<b>ensures</b> result.account_address == sender_addr;
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr);
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap_field">spec_get_key_rotation_cap_field</a>(sender_addr) == <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap_field">spec_get_key_rotation_cap_field</a>(sender_addr));
</code></pre>




<a name="0x1_LibraAccount_ExtractWithdrawCapAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_ExtractWithdrawCapAbortsIf">ExtractWithdrawCapAbortsIf</a> {
    sender_addr: address;
    <b>aborts_if</b> <a href="LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(sender_addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
    <b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(sender_addr) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_restore_withdraw_capability"></a>

## Function `restore_withdraw_capability`

Return the withdraw capability to the account it originally came from


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">restore_withdraw_capability</a>(cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>)
<b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(cap.account_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    // Abort <b>if</b> the withdraw capability for this account is not extracted,
    // indicating that the withdraw capability is not unique.
    <b>assert</b>(
        <a href="LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(cap.account_address),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="LibraAccount.md#0x1_LibraAccount_EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED">EWITHDRAWAL_CAPABILITY_NOT_EXTRACTED</a>)
    );
    <b>let</b> account = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.withdrawal_capability, cap)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<a name="0x1_LibraAccount_cap_addr$87"></a>
<b>let</b> cap_addr = cap.account_address;
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(cap_addr);
<b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(cap_addr) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
<b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(cap_addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_withdraw_cap">spec_holds_own_withdraw_cap</a>(cap_addr);
</code></pre>



</details>

<a name="0x1_LibraAccount_pay_from"></a>

## Function `pay_from`

Withdraw <code>amount</code> Libra<Token> from the address embedded in <code><a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code> and
deposits it into the <code>payee</code>'s account balance.
The included <code>metadata</code> will appear in the <code><a href="LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a></code> and <code><a href="LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a></code>.
The <code>metadata_signature</code> will only be checked if this payment is subject to the dual
attestation protocol


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_pay_from">pay_from</a>&lt;Token&gt;(cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>, payee: address, amount: u64, metadata: vector&lt;u8&gt;, metadata_signature: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_pay_from">pay_from</a>&lt;Token&gt;(
    cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>,
    payee: address,
    amount: u64,
    metadata: vector&lt;u8&gt;,
    metadata_signature: vector&lt;u8&gt;
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>, <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="LibraAccount.md#0x1_LibraAccount_deposit">deposit</a>&lt;Token&gt;(
        *&cap.account_address,
        payee,
        <a href="LibraAccount.md#0x1_LibraAccount_withdraw_from">withdraw_from</a>(cap, payee, amount, <b>copy</b> metadata),
        metadata,
        metadata_signature
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code>pragma opaque;
<a name="0x1_LibraAccount_payer$88"></a>
<b>let</b> payer = cap.account_address;
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payee);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer);
<b>modifies</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee);
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(payer);
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(payee);
<b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer);
<b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee);
<b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer).withdrawal_capability ==
    <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(payer).withdrawal_capability);
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_PayFromAbortsIf">PayFromAbortsIf</a>&lt;Token&gt;;
<b>ensures</b> payer == payee ==&gt; <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(payer) == <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(payer));
<b>ensures</b> payer != payee ==&gt; <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(payer) == <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(payer)) - amount;
<b>ensures</b> payer != payee ==&gt; <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(payee) == <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(payee)) + amount;
</code></pre>




<a name="0x1_LibraAccount_PayFromAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_PayFromAbortsIf">PayFromAbortsIf</a>&lt;Token&gt; {
    cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>;
    payee: address;
    amount: u64;
    metadata: vector&lt;u8&gt;;
    metadata_signature: vector&lt;u8&gt; ;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIf">DepositAbortsIf</a>&lt;Token&gt;{payer: cap.account_address};
    <b>aborts_if</b> cap.account_address != payee &&
        <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payee).coin.value + amount &gt; max_u64() <b>with</b> <a href="Errors.md#0x1_Errors_LIMIT_EXCEEDED">Errors::LIMIT_EXCEEDED</a>;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromAbortsIf">WithdrawFromAbortsIf</a>&lt;Token&gt;;
}
</code></pre>




<a name="0x1_LibraAccount_PayFromAbortsIfRestricted"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_PayFromAbortsIfRestricted">PayFromAbortsIfRestricted</a>&lt;Token&gt; {
    cap: <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>;
    payee: address;
    amount: u64;
    metadata: vector&lt;u8&gt;;
    metadata_signature: vector&lt;u8&gt; ;
    <a name="0x1_LibraAccount_payer$74"></a>
    <b>let</b> payer = cap.account_address;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_DepositAbortsIfRestricted">DepositAbortsIfRestricted</a>&lt;Token&gt;{payer: cap.account_address};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_WithdrawFromBalanceNoLimitsAbortsIf">WithdrawFromBalanceNoLimitsAbortsIf</a>&lt;Token&gt;{payer: payer, balance: <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer)};
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(payer) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_rotate_authentication_key"></a>

## Function `rotate_authentication_key`

Rotate the authentication key for the account under cap.account_address


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>, new_authentication_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_rotate_authentication_key">rotate_authentication_key</a>(
    cap: &<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>,
    new_authentication_key: vector&lt;u8&gt;,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>  {
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(cap.account_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <b>let</b> sender_account_resource = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    // Don't allow rotating <b>to</b> clearly invalid key
    <b>assert</b>(
        <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_authentication_key) == 32,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>)
    );
    sender_account_resource.authentication_key = new_authentication_key;
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyAbortsIf">RotateAuthenticationKeyAbortsIf</a>;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyEnsures">RotateAuthenticationKeyEnsures</a>{addr: cap.account_address};
</code></pre>


Can only rotate the authentication_key of cap.account_address [H16].


<pre><code><b>ensures</b> <b>forall</b> addr1: address <b>where</b> addr1 != cap.account_address && <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1)):
    <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1).authentication_key == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1).authentication_key);
</code></pre>




<a name="0x1_LibraAccount_RotateAuthenticationKeyAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyAbortsIf">RotateAuthenticationKeyAbortsIf</a> {
    cap: &<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>;
    new_authentication_key: vector&lt;u8&gt;;
    <b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(cap.account_address) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <b>aborts_if</b> len(new_authentication_key) != 32 <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>




<a name="0x1_LibraAccount_RotateAuthenticationKeyEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_RotateAuthenticationKeyEnsures">RotateAuthenticationKeyEnsures</a> {
    addr: address;
    new_authentication_key: vector&lt;u8&gt;;
    <b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr).authentication_key == new_authentication_key;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_extract_key_rotation_capability"></a>

## Function `extract_key_rotation_capability`

Return a unique capability granting permission to rotate the sender's authentication key


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_extract_key_rotation_capability">extract_key_rotation_capability</a>(account: &signer): <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>
<b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    // Abort <b>if</b> we already extracted the unique key rotation capability for this account.
    <b>assert</b>(
        !<a href="LibraAccount.md#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(account_address),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="LibraAccount.md#0x1_LibraAccount_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a>)
    );
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(account_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <b>let</b> account = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(account_address);
    <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> account.key_rotation_capability)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">ExtractKeyRotationCapabilityAbortsIf</a>;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityEnsures">ExtractKeyRotationCapabilityEnsures</a>;
</code></pre>




<a name="0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityAbortsIf">ExtractKeyRotationCapabilityAbortsIf</a> {
    account: signer;
    <a name="0x1_LibraAccount_account_addr$75"></a>
    <b>let</b> account_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(account_addr) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <b>aborts_if</b> <a href="LibraAccount.md#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(account_addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
}
</code></pre>




<a name="0x1_LibraAccount_ExtractKeyRotationCapabilityEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_ExtractKeyRotationCapabilityEnsures">ExtractKeyRotationCapabilityEnsures</a> {
    account: signer;
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
}
</code></pre>



</details>

<a name="0x1_LibraAccount_restore_key_rotation_capability"></a>

## Function `restore_key_rotation_capability`

Return the key rotation capability to the account it originally came from


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_restore_key_rotation_capability">restore_key_rotation_capability</a>(cap: <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>)
<b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(cap.account_address), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <b>let</b> account = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(cap.account_address);
    <a href="Option.md#0x1_Option_fill">Option::fill</a>(&<b>mut</b> account.key_rotation_capability, cap)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_RestoreKeyRotationCapabilityAbortsIf">RestoreKeyRotationCapabilityAbortsIf</a>;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_RestoreKeyRotationCapabilityEnsures">RestoreKeyRotationCapabilityEnsures</a>;
</code></pre>




<a name="0x1_LibraAccount_RestoreKeyRotationCapabilityAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_RestoreKeyRotationCapabilityAbortsIf">RestoreKeyRotationCapabilityAbortsIf</a> {
    cap: <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>;
    <b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(cap.account_address) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(cap.account_address) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>




<a name="0x1_LibraAccount_RestoreKeyRotationCapabilityEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_RestoreKeyRotationCapabilityEnsures">RestoreKeyRotationCapabilityEnsures</a> {
    cap: <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>;
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_key_rotation_cap">spec_holds_own_key_rotation_cap</a>(cap.account_address);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_add_currencies_for_account"></a>

## Function `add_currencies_for_account`

Add balances for <code>Token</code> to <code>new_account</code>.  If <code>add_all_currencies</code> is true,
then add for each of the three token types.
It is important that this be a private function. Otherwise, balances could
be added to inappropriate accounts. See invariant, "Only reasonable accounts
have currencies", below.


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(new_account: &signer, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(
    new_account: &signer,
    add_all_currencies: bool,
) {
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account);
    <a href="LibraAccount.md#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(new_account);
    <b>if</b> (add_all_currencies) {
        <b>if</b> (!<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(new_account_addr)) {
            <a href="LibraAccount.md#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;(new_account);
        };
        <b>if</b> (!<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(new_account_addr)) {
            <a href="LibraAccount.md#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;(new_account);
        };
        <b>if</b> (!<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(new_account_addr)) {
            <a href="LibraAccount.md#0x1_LibraAccount_add_currency">add_currency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;(new_account);
        };
    };
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<a name="0x1_LibraAccount_new_account_addr$89"></a>


<pre><code><b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(new_account);
<b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_can_hold_balance_addr">Roles::spec_can_hold_balance_addr</a>(new_account_addr) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountAbortsIf">AddCurrencyForAccountAbortsIf</a>&lt;Token&gt;{addr: new_account_addr};
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountEnsures">AddCurrencyForAccountEnsures</a>&lt;Token&gt;{addr: new_account_addr};
</code></pre>




<a name="0x1_LibraAccount_AddCurrencyForAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountAbortsIf">AddCurrencyForAccountAbortsIf</a>&lt;Token&gt; {
    addr: address;
    add_all_currencies: bool;
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;Token&gt;;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>include</b> add_all_currencies && !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(addr)
        ==&gt; <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;;
    <b>include</b> add_all_currencies && !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(addr)
        ==&gt; <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;;
    <b>include</b> add_all_currencies && !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(addr)
        ==&gt; <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;;
}
</code></pre>




<a name="0x1_LibraAccount_AddCurrencyForAccountEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountEnsures">AddCurrencyForAccountEnsures</a>&lt;Token&gt; {
    addr: address;
    add_all_currencies: bool;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">AddCurrencyEnsures</a>&lt;Token&gt;;
    <b>include</b> add_all_currencies && !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;&gt;(addr)
        ==&gt; <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">AddCurrencyEnsures</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;;
    <b>include</b> add_all_currencies && !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;&gt;(addr)
        ==&gt; <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">AddCurrencyEnsures</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;;
    <b>include</b> add_all_currencies && !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;&gt;(addr)
        ==&gt; <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">AddCurrencyEnsures</a>&lt;<a href="LBR.md#0x1_LBR">LBR</a>&gt;;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_make_account"></a>

## Function `make_account`

Creates a new account with account at <code>new_account_address</code> with
authentication key <code>auth_key_prefix</code> | <code>fresh_address</code>.
Aborts if there is already an account at <code>new_account_address</code>.

Creating an account at address 0x0 will abort as it is a reserved address for the MoveVM.


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(new_account: signer, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(
    new_account: signer,
    auth_key_prefix: vector&lt;u8&gt;,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    // Magically create a signer so we can do `move_to` <b>to</b> store <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, etc.
    // at the new address.
    <b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&new_account);
    // cannot create an account at the reserved address 0x0
    <b>assert</b>(
        new_account_addr != <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_ECANNOT_CREATE_AT_VM_RESERVED">ECANNOT_CREATE_AT_VM_RESERVED</a>)
    );

    // Construct authentication key.
    <b>let</b> authentication_key = auth_key_prefix;
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(
        &<b>mut</b> authentication_key, <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(<a href="Signer.md#0x1_Signer_borrow_address">Signer::borrow_address</a>(&new_account))
    );
    <b>assert</b>(
        <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&authentication_key) == 32,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_EMALFORMED_AUTHENTICATION_KEY">EMALFORMED_AUTHENTICATION_KEY</a>)
    );
    // Publish <a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">AccountFreezing::FreezingBit</a> (initially not frozen)
    <a href="AccountFreezing.md#0x1_AccountFreezing_create">AccountFreezing::create</a>(&new_account);
    // The <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> is published during <a href="Genesis.md#0x1_Genesis">Genesis</a>, so it should
    // always exist.  This is a sanity check.
    <b>assert</b>(
        <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT_OPERATIONS_CAPABILITY">EACCOUNT_OPERATIONS_CAPABILITY</a>)
    );
    // Emit the <a href="LibraAccount.md#0x1_LibraAccount_CreateAccountEvent">CreateAccountEvent</a>
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).creation_events,
        <a href="LibraAccount.md#0x1_LibraAccount_CreateAccountEvent">CreateAccountEvent</a> { created: new_account_addr, role_id: <a href="Roles.md#0x1_Roles_get_role_id">Roles::get_role_id</a>(new_account_addr) },
    );
    // Publishing the account <b>resource</b> last makes it possible <b>to</b> prove invariants that simplify
    // <b>aborts_if</b>'s, etc.
    move_to(
        &new_account,
        <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
            authentication_key,
            withdrawal_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
                <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a> {
                    account_address: new_account_addr
            }),
            key_rotation_capability: <a href="Option.md#0x1_Option_some">Option::some</a>(
                <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a> {
                    account_address: new_account_addr
            }),
            received_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_ReceivedPaymentEvent">ReceivedPaymentEvent</a>&gt;(&new_account),
            sent_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_SentPaymentEvent">SentPaymentEvent</a>&gt;(&new_account),
            sequence_number: 0,
        }
    );
    <a href="LibraAccount.md#0x1_LibraAccount_destroy_signer">destroy_signer</a>(new_account);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<a name="0x1_LibraAccount_new_account_addr$90"></a>


<pre><code><b>let</b> new_account_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(new_account);
<b>requires</b> <b>exists</b>&lt;<a href="Roles.md#0x1_Roles_RoleId">Roles::RoleId</a>&gt;(new_account_addr);
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_MakeAccountAbortsIf">MakeAccountAbortsIf</a>{addr: new_account_addr};
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(new_account_addr);
</code></pre>




<a name="0x1_LibraAccount_MakeAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_MakeAccountAbortsIf">MakeAccountAbortsIf</a> {
    addr: address;
    auth_key_prefix: vector&lt;u8&gt;;
    <b>aborts_if</b> addr == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>() <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">AccountFreezing::FreezingBit</a>&gt;(addr) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>aborts_if</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>() && !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
        <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
    <b>aborts_if</b> <a href="Vector.md#0x1_Vector_length">Vector::length</a>(auth_key_prefix) + <a href="Vector.md#0x1_Vector_length">Vector::length</a>(<a href="LCS.md#0x1_LCS_serialize">LCS::serialize</a>(addr)) != 32
        <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_libra_root_account"></a>

## Function `create_libra_root_account`

Creates the libra root account (during genesis). Publishes the Libra root role,
Publishes a SlidingNonce resource, sets up event generator, publishes
AccountOperationsCapability, WriteSetManager, and finally makes the account.


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_libra_root_account">create_libra_root_account</a>(auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_libra_root_account">create_libra_root_account</a>(
    auth_key_prefix: vector&lt;u8&gt;,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <b>let</b> lr_account = <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(&lr_account);
    <a href="Roles.md#0x1_Roles_grant_libra_root_role">Roles::grant_libra_root_role</a>(&lr_account);
    <a href="SlidingNonce.md#0x1_SlidingNonce_publish_nonce_resource">SlidingNonce::publish_nonce_resource</a>(&lr_account, &lr_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&lr_account);

    <b>assert</b>(
        !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT_OPERATIONS_CAPABILITY">EACCOUNT_OPERATIONS_CAPABILITY</a>)
    );
    move_to(
        &lr_account,
        <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
            limits_cap: <a href="AccountLimits.md#0x1_AccountLimits_grant_mutation_capability">AccountLimits::grant_mutation_capability</a>(&lr_account),
            creation_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_CreateAccountEvent">CreateAccountEvent</a>&gt;(&lr_account),
        }
    );
    <b>assert</b>(
        !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EWRITESET_MANAGER">EWRITESET_MANAGER</a>)
    );
    move_to(
        &lr_account,
        <a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a> {
            upgrade_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_UpgradeEvent">Self::UpgradeEvent</a>&gt;(&lr_account),
        }
    );

    <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(lr_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_treasury_compliance_account"></a>

## Function `create_treasury_compliance_account`

Create a treasury/compliance account at <code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> | <code>new_account_address</code>.  Can only be called during genesis.
Also, publishes the treasury compliance role, the SlidingNonce resource, and
event handle generator, then makes the account.


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>(lr_account: &signer, auth_key_prefix: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_treasury_compliance_account">create_treasury_compliance_account</a>(
    lr_account: &signer,
    auth_key_prefix: vector&lt;u8&gt;,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <b>let</b> new_account_address = <a href="CoreAddresses.md#0x1_CoreAddresses_TREASURY_COMPLIANCE_ADDRESS">CoreAddresses::TREASURY_COMPLIANCE_ADDRESS</a>();
    <b>let</b> new_account = <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Roles.md#0x1_Roles_grant_treasury_compliance_role">Roles::grant_treasury_compliance_role</a>(&new_account, lr_account);
    <a href="SlidingNonce.md#0x1_SlidingNonce_publish_nonce_resource">SlidingNonce::publish_nonce_resource</a>(lr_account, &new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_designated_dealer"></a>

## Function `create_designated_dealer`

Create a designated dealer account at <code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> | <code>new_account_address</code>, for non synthetic CoinType.
Creates Preburn resource under account 'new_account_address'


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(creator_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_designated_dealer">create_designated_dealer</a>&lt;CoinType&gt;(
    creator_account: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    add_all_currencies: bool,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> new_dd_account = <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_dd_account);
    <a href="Roles.md#0x1_Roles_new_designated_dealer_role">Roles::new_designated_dealer_role</a>(creator_account, &new_dd_account);
    <a href="DesignatedDealer.md#0x1_DesignatedDealer_publish_designated_dealer_credential">DesignatedDealer::publish_designated_dealer_credential</a>&lt;CoinType&gt;(&new_dd_account, creator_account, add_all_currencies);
    <a href="LibraAccount.md#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;CoinType&gt;(&new_dd_account, add_all_currencies);
    <a href="DualAttestation.md#0x1_DualAttestation_publish_credential">DualAttestation::publish_credential</a>(&new_dd_account, creator_account, human_name);
    <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(new_dd_account, auth_key_prefix)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateDesignatedDealerAbortsIf">CreateDesignatedDealerAbortsIf</a>&lt;CoinType&gt;;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateDesignatedDealerEnsures">CreateDesignatedDealerEnsures</a>&lt;CoinType&gt;;
</code></pre>




<a name="0x1_LibraAccount_CreateDesignatedDealerAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateDesignatedDealerAbortsIf">CreateDesignatedDealerAbortsIf</a>&lt;CoinType&gt; {
    creator_account: signer;
    new_account_address: address;
    auth_key_prefix: vector&lt;u8&gt;;
    add_all_currencies: bool;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: creator_account};
    <b>aborts_if</b> <b>exists</b>&lt;<a href="Roles.md#0x1_Roles_RoleId">Roles::RoleId</a>&gt;(new_account_address) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>aborts_if</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>&gt;(new_account_address) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>include</b> <b>if</b> (add_all_currencies)
                <a href="DesignatedDealer.md#0x1_DesignatedDealer_AddCurrencyAbortsIf">DesignatedDealer::AddCurrencyAbortsIf</a>&lt;<a href="Coin1.md#0x1_Coin1">Coin1</a>&gt;{dd_addr: new_account_address}
                && <a href="DesignatedDealer.md#0x1_DesignatedDealer_AddCurrencyAbortsIf">DesignatedDealer::AddCurrencyAbortsIf</a>&lt;<a href="Coin2.md#0x1_Coin2">Coin2</a>&gt;{dd_addr: new_account_address}
            <b>else</b> <a href="DesignatedDealer.md#0x1_DesignatedDealer_AddCurrencyAbortsIf">DesignatedDealer::AddCurrencyAbortsIf</a>&lt;CoinType&gt;{dd_addr: new_account_address};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountAbortsIf">AddCurrencyForAccountAbortsIf</a>&lt;CoinType&gt;{addr: new_account_address};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_MakeAccountAbortsIf">MakeAccountAbortsIf</a>{addr: new_account_address};
}
</code></pre>




<a name="0x1_LibraAccount_CreateDesignatedDealerEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateDesignatedDealerEnsures">CreateDesignatedDealerEnsures</a>&lt;CoinType&gt; {
    new_account_address: address;
    <b>ensures</b> <b>exists</b>&lt;<a href="DesignatedDealer.md#0x1_DesignatedDealer_Dealer">DesignatedDealer::Dealer</a>&gt;(new_account_address);
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(new_account_address);
    <b>ensures</b> <a href="Roles.md#0x1_Roles_spec_has_designated_dealer_role_addr">Roles::spec_has_designated_dealer_role_addr</a>(new_account_address);
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountEnsures">AddCurrencyForAccountEnsures</a>&lt;CoinType&gt;{addr: new_account_address};
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_parent_vasp_account"></a>

## Function `create_parent_vasp_account`

Create an account with the ParentVASP role at <code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> | <code>new_account_address</code>.  If <code>add_all_currencies</code> is true, 0 balances for
all available currencies in the system will also be added.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_parent_vasp_account">create_parent_vasp_account</a>&lt;Token&gt;(creator_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_parent_vasp_account">create_parent_vasp_account</a>&lt;Token&gt;(
    creator_account: &signer,  // TreasuryCompliance
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
    add_all_currencies: bool
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> new_account = <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Roles.md#0x1_Roles_new_parent_vasp_role">Roles::new_parent_vasp_role</a>(creator_account, &new_account);
    <a href="VASP.md#0x1_VASP_publish_parent_vasp_credential">VASP::publish_parent_vasp_credential</a>(&new_account, creator_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="DualAttestation.md#0x1_DualAttestation_publish_credential">DualAttestation::publish_credential</a>(&new_account, creator_account, human_name);
    <a href="LibraAccount.md#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(&new_account, add_all_currencies);
    <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateParentVASPAccountAbortsIf">CreateParentVASPAccountAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateParentVASPAccountEnsures">CreateParentVASPAccountEnsures</a>&lt;Token&gt;;
</code></pre>




<a name="0x1_LibraAccount_CreateParentVASPAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateParentVASPAccountAbortsIf">CreateParentVASPAccountAbortsIf</a>&lt;Token&gt; {
    creator_account: signer;
    new_account_address: address;
    auth_key_prefix: vector&lt;u8&gt;;
    add_all_currencies: bool;
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotTreasuryCompliance">Roles::AbortsIfNotTreasuryCompliance</a>{account: creator_account};
    <b>aborts_if</b> <b>exists</b>&lt;<a href="Roles.md#0x1_Roles_RoleId">Roles::RoleId</a>&gt;(new_account_address) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>aborts_if</b> <a href="VASP.md#0x1_VASP_is_vasp">VASP::is_vasp</a>(new_account_address) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountAbortsIf">AddCurrencyForAccountAbortsIf</a>&lt;Token&gt;{addr: new_account_address};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_MakeAccountAbortsIf">MakeAccountAbortsIf</a>{addr: new_account_address};
}
</code></pre>




<a name="0x1_LibraAccount_CreateParentVASPAccountEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateParentVASPAccountEnsures">CreateParentVASPAccountEnsures</a>&lt;Token&gt; {
    new_account_address: address;
    <b>include</b> <a href="VASP.md#0x1_VASP_PublishParentVASPEnsures">VASP::PublishParentVASPEnsures</a>{vasp_addr: new_account_address};
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(new_account_address);
    <b>ensures</b> <a href="Roles.md#0x1_Roles_spec_has_parent_VASP_role_addr">Roles::spec_has_parent_VASP_role_addr</a>(new_account_address);
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountEnsures">AddCurrencyForAccountEnsures</a>&lt;Token&gt;{addr: new_account_address};
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_child_vasp_account"></a>

## Function `create_child_vasp_account`

Create an account with the ChildVASP role at <code>new_account_address</code> with authentication key
<code>auth_key_prefix</code> | <code>new_account_address</code> and a 0 balance of type <code>Token</code>. If
<code>add_all_currencies</code> is true, 0 balances for all avaialable currencies in the system will
also be added. This account will be a child of <code>creator</code>, which must be a ParentVASP.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_child_vasp_account">create_child_vasp_account</a>&lt;Token&gt;(parent: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, add_all_currencies: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_child_vasp_account">create_child_vasp_account</a>&lt;Token&gt;(
    parent: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    add_all_currencies: bool,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> new_account = <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    <a href="Roles.md#0x1_Roles_new_child_vasp_role">Roles::new_child_vasp_role</a>(parent, &new_account);
    <a href="VASP.md#0x1_VASP_publish_child_vasp_credential">VASP::publish_child_vasp_credential</a>(
        parent,
        &new_account,
    );
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="LibraAccount.md#0x1_LibraAccount_add_currencies_for_account">add_currencies_for_account</a>&lt;Token&gt;(&new_account, add_all_currencies);
    <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateChildVASPAccountAbortsIf">CreateChildVASPAccountAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateChildVASPAccountEnsures">CreateChildVASPAccountEnsures</a>&lt;Token&gt;{
    parent_addr: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(parent),
    child_addr: new_account_address,
};
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountEnsures">AddCurrencyForAccountEnsures</a>&lt;Token&gt;{addr: new_account_address};
</code></pre>




<a name="0x1_LibraAccount_CreateChildVASPAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateChildVASPAccountAbortsIf">CreateChildVASPAccountAbortsIf</a>&lt;Token&gt; {
    parent: signer;
    new_account_address: address;
    auth_key_prefix: vector&lt;u8&gt;;
    add_all_currencies: bool;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotParentVasp">Roles::AbortsIfNotParentVasp</a>{account: parent};
    <b>aborts_if</b> <b>exists</b>&lt;<a href="Roles.md#0x1_Roles_RoleId">Roles::RoleId</a>&gt;(new_account_address) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
    <b>include</b> <a href="VASP.md#0x1_VASP_PublishChildVASPAbortsIf">VASP::PublishChildVASPAbortsIf</a>{child_addr: new_account_address};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyForAccountAbortsIf">AddCurrencyForAccountAbortsIf</a>&lt;Token&gt;{addr: new_account_address};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_MakeAccountAbortsIf">MakeAccountAbortsIf</a>{addr: new_account_address};
}
</code></pre>




<a name="0x1_LibraAccount_CreateChildVASPAccountEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateChildVASPAccountEnsures">CreateChildVASPAccountEnsures</a>&lt;Token&gt; {
    parent_addr: address;
    child_addr: address;
    add_all_currencies: bool;
    <b>include</b> <a href="VASP.md#0x1_VASP_PublishChildVASPEnsures">VASP::PublishChildVASPEnsures</a>;
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(child_addr);
    <b>ensures</b> <a href="Roles.md#0x1_Roles_spec_has_child_VASP_role_addr">Roles::spec_has_child_VASP_role_addr</a>(child_addr);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_signer"></a>

## Function `create_signer`



<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(addr: address): signer
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(addr: address): signer;
</code></pre>



</details>

<a name="0x1_LibraAccount_destroy_signer"></a>

## Function `destroy_signer`



<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_destroy_signer">destroy_signer</a>(sig: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_destroy_signer">destroy_signer</a>(sig: signer);
</code></pre>



</details>

<a name="0x1_LibraAccount_balance_for"></a>

## Function `balance_for`

Helper to return the u64 value of the <code>balance</code> for <code>account</code>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_balance_for">balance_for</a>&lt;Token&gt;(balance: &<a href="LibraAccount.md#0x1_LibraAccount_Balance">LibraAccount::Balance</a>&lt;Token&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_balance_for">balance_for</a>&lt;Token&gt;(balance: &<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;): u64 {
    <a href="Libra.md#0x1_Libra_value">Libra::value</a>&lt;Token&gt;(&balance.coin)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_balance"></a>

## Function `balance`

Return the current balance of the account at <code>addr</code>.


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(addr: address): u64 <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a> {
    <b>assert</b>(<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EPAYER_DOESNT_HOLD_CURRENCY">EPAYER_DOESNT_HOLD_CURRENCY</a>));
    <a href="LibraAccount.md#0x1_LibraAccount_balance_for">balance_for</a>(borrow_global&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr))
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr) <b>with</b> <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>



</details>

<a name="0x1_LibraAccount_add_currency"></a>

## Function `add_currency`

Add a balance of <code>Token</code> type to the sending account


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_add_currency">add_currency</a>&lt;Token&gt;(account: &signer) {
    // aborts <b>if</b> `Token` is not a currency type in the system
    <a href="Libra.md#0x1_Libra_assert_is_currency">Libra::assert_is_currency</a>&lt;Token&gt;();
    // Check that an account <b>with</b> this role is allowed <b>to</b> hold funds
    <b>assert</b>(
        <a href="Roles.md#0x1_Roles_can_hold_balance">Roles::can_hold_balance</a>(account),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_EROLE_CANT_STORE_BALANCE">EROLE_CANT_STORE_BALANCE</a>)
    );
    // aborts <b>if</b> this account already has a balance in `Token`
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(!<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EADD_EXISTING_CURRENCY">EADD_EXISTING_CURRENCY</a>));

    move_to(account, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;{ coin: <a href="Libra.md#0x1_Libra_zero">Libra::zero</a>&lt;Token&gt;() })
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;Token&gt;;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">AddCurrencyEnsures</a>&lt;Token&gt;{addr: <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)};
</code></pre>




<a name="0x1_LibraAccount_AddCurrencyAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;Token&gt; {
    account: signer;
}
</code></pre>


<code>Currency</code> must be valid


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;Token&gt; {
    <b>include</b> <a href="Libra.md#0x1_Libra_AbortsIfNoCurrency">Libra::AbortsIfNoCurrency</a>&lt;Token&gt;;
}
</code></pre>


<code>account</code> must be allowed to hold balances. This function must abort if the predicate
<code>can_hold_balance</code> for <code>account</code> returns false [D1][D2][D3][D4][D5][D6][D7].


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;Token&gt; {
    <b>aborts_if</b> !<a href="Roles.md#0x1_Roles_can_hold_balance">Roles::can_hold_balance</a>(account) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


<code>account</code> cannot have an existing balance in <code>Currency</code>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyAbortsIf">AddCurrencyAbortsIf</a>&lt;Token&gt; {
    <b>aborts_if</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
}
</code></pre>




<a name="0x1_LibraAccount_AddCurrencyEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">AddCurrencyEnsures</a>&lt;Token&gt; {
    addr: address;
}
</code></pre>


This publishes a <code><a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Currency&gt;</code> to the caller's account


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AddCurrencyEnsures">AddCurrencyEnsures</a>&lt;Token&gt; {
    <b>ensures</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr);
    <b>ensures</b> <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr)
        == <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;{ coin: <a href="Libra.md#0x1_Libra">Libra</a>&lt;Token&gt; { value: 0 } };
}
</code></pre>



</details>

<a name="0x1_LibraAccount_accepts_currency"></a>

## Function `accepts_currency`

Return whether the account at <code>addr</code> accepts <code>Token</code> type coins


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_accepts_currency">accepts_currency</a>&lt;Token&gt;(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_accepts_currency">accepts_currency</a>&lt;Token&gt;(addr: address): bool {
    <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_sequence_number_for_account"></a>

## Function `sequence_number_for_account`

Helper to return the sequence number field for given <code>account</code>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="LibraAccount.md#0x1_LibraAccount_LibraAccount">LibraAccount::LibraAccount</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(account: &<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>): u64 {
    account.sequence_number
}
</code></pre>



</details>

<a name="0x1_LibraAccount_sequence_number"></a>

## Function `sequence_number`

Return the current sequence number at <code>addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_sequence_number">sequence_number</a>(addr: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_sequence_number">sequence_number</a>(addr: address): u64 <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <a href="LibraAccount.md#0x1_LibraAccount_sequence_number_for_account">sequence_number_for_account</a>(borrow_global&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr))
}
</code></pre>



</details>

<a name="0x1_LibraAccount_authentication_key"></a>

## Function `authentication_key`

Return the authentication key for this account


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_authentication_key">authentication_key</a>(addr: address): vector&lt;u8&gt; <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    *&borrow_global&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr).authentication_key
}
</code></pre>



</details>

<a name="0x1_LibraAccount_delegated_key_rotation_capability"></a>

## Function `delegated_key_rotation_capability`

Return true if the account at <code>addr</code> has delegated its key rotation capability


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr: address): bool
<b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr).key_rotation_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_delegated_withdraw_capability"></a>

## Function `delegated_withdraw_capability`

Return true if the account at <code>addr</code> has delegated its withdraw capability


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr: address): bool
<b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a> {
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&borrow_global&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr).withdrawal_capability)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_withdraw_capability_address"></a>

## Function `withdraw_capability_address`

Return a reference to the address associated with the given withdraw capability


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">LibraAccount::WithdrawCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_withdraw_capability_address">withdraw_capability_address</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_LibraAccount_key_rotation_capability_address"></a>

## Function `key_rotation_capability_address`

Return a reference to the address associated with the given key rotation capability


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">LibraAccount::KeyRotationCapability</a>): &address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_key_rotation_capability_address">key_rotation_capability_address</a>(cap: &<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>): &address {
    &cap.account_address
}
</code></pre>



</details>

<a name="0x1_LibraAccount_exists_at"></a>

## Function `exists_at`

Checks if an account exists at <code>check_addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(check_addr: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(check_addr: address): bool {
    <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(check_addr)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_module_prologue"></a>

## Function `module_prologue`

The prologue for module transaction


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_module_prologue">module_prologue</a>&lt;Token&gt;(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_module_prologue">module_prologue</a>&lt;Token&gt;(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
    chain_id: u8,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a> {
    <b>assert</b>(
        <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_module_allowed">LibraTransactionPublishingOption::is_module_allowed</a>(sender),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EMODULE_NOT_ALLOWED">PROLOGUE_EMODULE_NOT_ALLOWED</a>),
    );

    <a href="LibraAccount.md#0x1_LibraAccount_prologue_common">prologue_common</a>&lt;Token&gt;(
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

<details>
<summary>Specification</summary>



<a name="0x1_LibraAccount_transaction_sender$91"></a>


<pre><code><b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender);
<a name="0x1_LibraAccount_max_transaction_fee$92"></a>
<b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfModulePrologue">AbortsIfModulePrologue</a>&lt;Token&gt; {
    sender,
    txn_sequence_number,
    txn_public_key,
    chain_id,
    max_transaction_fee,
    txn_expiration_time_seconds: txn_expiration_time,
};
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_prologue_guarantees">prologue_guarantees</a>(sender);
</code></pre>




<a name="0x1_LibraAccount_AbortsIfModulePrologue"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfModulePrologue">AbortsIfModulePrologue</a>&lt;Token&gt; {
    sender: signer;
    txn_sequence_number: u64;
    txn_public_key: vector&lt;u8&gt;;
    chain_id: u8;
    max_transaction_fee: u128;
    txn_expiration_time_seconds: u64;
    <a name="0x1_LibraAccount_transaction_sender$76"></a>
    <b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender);
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
        transaction_sender,
        txn_sequence_number,
        txn_public_key,
        chain_id,
        max_transaction_fee,
        txn_expiration_time_seconds,
    };
}
</code></pre>


Aborts only in genesis. Does not need to be handled.


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfModulePrologue">AbortsIfModulePrologue</a>&lt;Token&gt; {
    <b>include</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">LibraTransactionPublishingOption::AbortsIfNoTransactionPublishingOption</a>;
}
</code></pre>


Covered: L75 (Match 9)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfModulePrologue">AbortsIfModulePrologue</a>&lt;Token&gt; {
    <b>aborts_if</b> !<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_spec_is_module_allowed">LibraTransactionPublishingOption::spec_is_module_allowed</a>(sender) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_script_prologue"></a>

## Function `script_prologue`

The prologue for script transaction


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_script_prologue">script_prologue</a>&lt;Token&gt;(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, chain_id: u8, script_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_script_prologue">script_prologue</a>&lt;Token&gt;(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
    chain_id: u8,
    script_hash: vector&lt;u8&gt;,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a> {
    <b>assert</b>(
        <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_script_allowed">LibraTransactionPublishingOption::is_script_allowed</a>(sender, &script_hash),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESCRIPT_NOT_ALLOWED">PROLOGUE_ESCRIPT_NOT_ALLOWED</a>),
    );

    <a href="LibraAccount.md#0x1_LibraAccount_prologue_common">prologue_common</a>&lt;Token&gt;(
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

<details>
<summary>Specification</summary>



<a name="0x1_LibraAccount_transaction_sender$93"></a>


<pre><code><b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender);
<a name="0x1_LibraAccount_max_transaction_fee$94"></a>
<b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfScriptPrologue">AbortsIfScriptPrologue</a>&lt;Token&gt;{
    max_transaction_fee,
    txn_expiration_time_seconds: txn_expiration_time,
};
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_prologue_guarantees">prologue_guarantees</a>(sender);
</code></pre>




<a name="0x1_LibraAccount_AbortsIfScriptPrologue"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfScriptPrologue">AbortsIfScriptPrologue</a>&lt;Token&gt; {
    sender: signer;
    txn_sequence_number: u64;
    txn_public_key: vector&lt;u8&gt;;
    chain_id: u8;
    max_transaction_fee: u128;
    txn_expiration_time_seconds: u64;
    script_hash: vector&lt;u8&gt;;
    <a name="0x1_LibraAccount_transaction_sender$77"></a>
    <b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender);
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {transaction_sender};
}
</code></pre>


Aborts only in Genesis. Does not need to be handled.


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfScriptPrologue">AbortsIfScriptPrologue</a>&lt;Token&gt; {
    <b>include</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">LibraTransactionPublishingOption::AbortsIfNoTransactionPublishingOption</a>;
}
</code></pre>


Covered: L74 (Match 8)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfScriptPrologue">AbortsIfScriptPrologue</a>&lt;Token&gt; {
    <b>aborts_if</b> !<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_spec_is_script_allowed">LibraTransactionPublishingOption::spec_is_script_allowed</a>(sender, script_hash) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
}
</code></pre>




<a name="0x1_LibraAccount_prologue_guarantees"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_prologue_guarantees">prologue_guarantees</a>(sender: signer) : bool {
<b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender);
<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() && <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr) && !<a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(addr)
}
</code></pre>


Used in transaction script to specify properties checked by the prologue.


<a name="0x1_LibraAccount_TransactionChecks"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_TransactionChecks">TransactionChecks</a> {
    sender: signer;
    <b>requires</b> <a href="LibraAccount.md#0x1_LibraAccount_prologue_guarantees">prologue_guarantees</a>(sender);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_writeset_prologue"></a>

## Function `writeset_prologue`

The prologue for WriteSet transaction


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_writeset_prologue">writeset_prologue</a>(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_expiration_time: u64, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_writeset_prologue">writeset_prologue</a>(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_expiration_time: u64,
    chain_id: u8,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a> {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_INVALID_WRITESET_SENDER">PROLOGUE_INVALID_WRITESET_SENDER</a>)
    );
    <b>assert</b>(<a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(sender), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_INVALID_WRITESET_SENDER">PROLOGUE_INVALID_WRITESET_SENDER</a>));

    // Currency code don't matter here <b>as</b> it won't be charged anyway. Gas constants are ommitted.
    <a href="LibraAccount.md#0x1_LibraAccount_prologue_common">prologue_common</a>&lt;<a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;(
        sender,
        txn_sequence_number,
        txn_public_key,
        0,
        0,
        txn_expiration_time,
        chain_id,
    )
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfWritesetPrologue">AbortsIfWritesetPrologue</a> {txn_expiration_time_seconds: txn_expiration_time};
<b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_prologue_guarantees">prologue_guarantees</a>(sender);
</code></pre>




<a name="0x1_LibraAccount_AbortsIfWritesetPrologue"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfWritesetPrologue">AbortsIfWritesetPrologue</a> {
    sender: signer;
    txn_sequence_number: u64;
    txn_public_key: vector&lt;u8&gt;;
    txn_expiration_time_seconds: u64;
    chain_id: u8;
    <a name="0x1_LibraAccount_transaction_sender$78"></a>
    <b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender);
}
</code></pre>


Covered: L146 (Match 0)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfWritesetPrologue">AbortsIfWritesetPrologue</a> {
    <b>aborts_if</b> transaction_sender != <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>() <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


Must abort if the signer does not have the LibraRoot role [H8].
Covered: L146 (Match 0)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfWritesetPrologue">AbortsIfWritesetPrologue</a> {
    <b>aborts_if</b> !<a href="Roles.md#0x1_Roles_spec_has_libra_root_role_addr">Roles::spec_has_libra_root_role_addr</a>(transaction_sender) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;<a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;{
        transaction_sender,
        max_transaction_fee: 0,
    };
}
</code></pre>



</details>

<a name="0x1_LibraAccount_prologue_common"></a>

## Function `prologue_common`

The common prologue is invoked at the beginning of every transaction
The main properties that it verifies:
- The account's auth key matches the transaction's public key
- That the account has enough balance to pay for all of the gas
- That the sequence number matches the transaction's sequence key


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_prologue_common">prologue_common</a>&lt;Token&gt;(sender: &signer, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time_seconds: u64, chain_id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_prologue_common">prologue_common</a>&lt;Token&gt;(
    sender: &signer,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time_seconds: u64,
    chain_id: u8,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a> {
    <b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);

    // [PCA1]: Check that the chain ID stored on-chain matches the chain ID specified by the transaction
    <b>assert</b>(<a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() == chain_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EBAD_CHAIN_ID">PROLOGUE_EBAD_CHAIN_ID</a>));

    // [PCA2]: Verify that the transaction sender's account <b>exists</b>
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(transaction_sender), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EACCOUNT_DNE">PROLOGUE_EACCOUNT_DNE</a>));

    // [PCA3]: We check whether this account is frozen, <b>if</b> it is no transaction can be sent from it.
    <b>assert</b>(
        !<a href="AccountFreezing.md#0x1_AccountFreezing_account_is_frozen">AccountFreezing::account_is_frozen</a>(transaction_sender),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EACCOUNT_FROZEN">PROLOGUE_EACCOUNT_FROZEN</a>)
    );

    // Load the transaction sender's account
    <b>let</b> sender_account = borrow_global&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(transaction_sender);

    // [PCA4]: Check that the hash of the transaction's <b>public</b> key matches the account's auth key
    <b>assert</b>(
        <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) == *&sender_account.authentication_key,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY">PROLOGUE_EINVALID_ACCOUNT_AUTH_KEY</a>),
    );

    // [PCA5]: Check that the account has enough balance for all of the gas
    <b>assert</b>(
        (txn_gas_price <b>as</b> u128) * (txn_max_gas_units <b>as</b> u128) &lt;= <a href="LibraAccount.md#0x1_LibraAccount_MAX_U64">MAX_U64</a>,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ECANT_PAY_GAS_DEPOSIT">PROLOGUE_ECANT_PAY_GAS_DEPOSIT</a>),
    );

    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;

    // Don't grab the balance <b>if</b> the transaction fee is zero
    <b>if</b> (max_transaction_fee &gt; 0) {
        // [PCA6]: Check that the account has a balance in this currency
        <b>assert</b>(
            <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(transaction_sender),
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ECANT_PAY_GAS_DEPOSIT">PROLOGUE_ECANT_PAY_GAS_DEPOSIT</a>)
        );
        <b>let</b> balance_amount = <a href="LibraAccount.md#0x1_LibraAccount_balance">balance</a>&lt;Token&gt;(transaction_sender);
        // [PCA7]: Check that the account can cover the maximum transaction fee
        <b>assert</b>(
            balance_amount &gt;= max_transaction_fee,
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ECANT_PAY_GAS_DEPOSIT">PROLOGUE_ECANT_PAY_GAS_DEPOSIT</a>)
        );
    };

    // [PCA8]: Check that the transaction sequence number is not too <b>old</b> (in the past)
    <b>assert</b>(
        txn_sequence_number &gt;= sender_account.sequence_number,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD">PROLOGUE_ESEQUENCE_NUMBER_TOO_OLD</a>)
    );

    // [PCA9]: Check that the transaction's sequence number matches the
    // current sequence number. Otherwise sequence number is too new by [PCA8].
    <b>assert</b>(
        txn_sequence_number == sender_account.sequence_number,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW">PROLOGUE_ESEQUENCE_NUMBER_TOO_NEW</a>)
    );

    // [PCA10]: Check that the transaction hasn't expired
    <b>assert</b>(
        <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_seconds">LibraTimestamp::now_seconds</a>() &lt; txn_expiration_time_seconds,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ETRANSACTION_EXPIRED">PROLOGUE_ETRANSACTION_EXPIRED</a>)
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<a name="0x1_LibraAccount_transaction_sender$95"></a>


<pre><code><b>let</b> transaction_sender = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(sender);
<a name="0x1_LibraAccount_max_transaction_fee$96"></a>
<b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    transaction_sender,
    max_transaction_fee,
};
</code></pre>




<a name="0x1_LibraAccount_AbortsIfPrologueCommon"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    transaction_sender: address;
    txn_sequence_number: u64;
    txn_public_key: vector&lt;u8&gt;;
    chain_id: u8;
    max_transaction_fee: u128;
    txn_expiration_time_seconds: u64;
}
</code></pre>


Only happens if this is called in Genesis. Doesn't need to be handled.


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
}
</code></pre>


[PCA1] Covered: L73 (Match 7)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> chain_id != <a href="ChainId.md#0x1_ChainId_spec_get_chain_id">ChainId::spec_get_chain_id</a>() <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


[PCA2] Covered: L65 (Match 4)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> !<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(transaction_sender) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


[PCA3] Covered: L57 (Match 0)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> <a href="AccountFreezing.md#0x1_AccountFreezing_spec_account_is_frozen">AccountFreezing::spec_account_is_frozen</a>(transaction_sender) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>;
}
</code></pre>


[PCA4] Covered: L59 (Match 1)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) != <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(transaction_sender).authentication_key <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


[PCA5] Covered: L69 (Match 5)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> max_transaction_fee &gt; <a href="LibraAccount.md#0x1_LibraAccount_MAX_U64">MAX_U64</a> <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


[PCA6] Covered: L69 (Match 5)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> max_transaction_fee &gt; 0 && !<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(transaction_sender) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


[PCA7] Covered: L69 (Match 5)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> max_transaction_fee &gt; 0 && <a href="LibraAccount.md#0x1_LibraAccount_spec_get_balance_value">spec_get_balance_value</a>&lt;Token&gt;(transaction_sender) &lt; max_transaction_fee <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


[PCA8] Covered: L61 (Match 2)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> txn_sequence_number &lt; <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(transaction_sender).sequence_number <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


[PCA9] Covered: L63 (match 3)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> txn_sequence_number &gt; <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(transaction_sender).sequence_number <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>


[PCA10] Covered: L72 (Match 6)


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AbortsIfPrologueCommon">AbortsIfPrologueCommon</a>&lt;Token&gt; {
    <b>aborts_if</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_seconds">LibraTimestamp::spec_now_seconds</a>() &gt;= txn_expiration_time_seconds <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
}
</code></pre>



</details>

<a name="0x1_LibraAccount_epilogue"></a>

## Function `epilogue`

Collects gas and bumps the sequence number for executing a transaction.
The epilogue is invoked at the end of the transaction.
If the exection of the epilogue fails, it is re-invoked with different arguments, and
based on the conditions checked in the prologue, should never fail.


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(account: &signer, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_epilogue">epilogue</a>&lt;Token&gt;(
    account: &signer,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a> {
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);

    // [EA1; Invariant]: Make sure that the transaction's `max_gas_units` is greater
    // than the number of gas units remaining after execution.
    <b>assert</b>(txn_max_gas_units &gt;= gas_units_remaining, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_EGAS">EGAS</a>));
    <b>let</b> gas_used = txn_max_gas_units - gas_units_remaining;

    // [EA2; Invariant]: Make sure that the transaction fee would not overflow maximum
    // number representable in a u64. Already checked in [PCA5].
    <b>assert</b>(
        (txn_gas_price <b>as</b> u128) * (gas_used <b>as</b> u128) &lt;= <a href="LibraAccount.md#0x1_LibraAccount_MAX_U64">MAX_U64</a>,
        <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="LibraAccount.md#0x1_LibraAccount_EGAS">EGAS</a>)
    );
    <b>let</b> transaction_fee_amount = txn_gas_price * gas_used;

    // [EA3; Invariant]: Make sure that account <b>exists</b>, and load the
    // transaction sender's account. Already checked in [PCA2].
    <b>assert</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(sender), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="LibraAccount.md#0x1_LibraAccount_EACCOUNT">EACCOUNT</a>));
    <b>let</b> sender_account = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(sender);

    // [EA4; Condition]: Make sure account's sequence number is within the
    // representable range of u64. Bump the sequence number
    <b>assert</b>(
        sender_account.<a href="LibraAccount.md#0x1_LibraAccount_sequence_number">sequence_number</a> &lt; (<a href="LibraAccount.md#0x1_LibraAccount_MAX_U64">MAX_U64</a> <b>as</b> u64),
        <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="LibraAccount.md#0x1_LibraAccount_ESEQUENCE_NUMBER">ESEQUENCE_NUMBER</a>)
    );

    // [EA4; Invariant]: Make sure passed-in `txn_sequence_number` matches
    // the `sender_account`'s `sequence_number`. Already checked in [PCA9].
    <b>assert</b>(
        sender_account.sequence_number == txn_sequence_number,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraAccount.md#0x1_LibraAccount_ESEQUENCE_NUMBER">ESEQUENCE_NUMBER</a>)
    );

    // The transaction sequence number is passed in <b>to</b> prevent any
    // possibility of the account's sequence number increasing by more than
    // one for any transaction.
    sender_account.sequence_number = txn_sequence_number + 1;

    <b>if</b> (transaction_fee_amount &gt; 0) {
        // [Invariant Use]: <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a> for `Token` verified <b>to</b> exist for non-zero transaction fee amounts by [PCA6].
        <b>let</b> sender_balance = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(sender);
        <b>let</b> coin = &<b>mut</b> sender_balance.coin;

        // [EA4; Condition]: Abort <b>if</b> this withdrawal would make the `sender_account`'s balance go negative
        <b>assert</b>(
            transaction_fee_amount &lt;= <a href="Libra.md#0x1_Libra_value">Libra::value</a>(coin),
            <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="LibraAccount.md#0x1_LibraAccount_PROLOGUE_ECANT_PAY_GAS_DEPOSIT">PROLOGUE_ECANT_PAY_GAS_DEPOSIT</a>)
        );

        // NB: `withdraw_from_balance` is not used <b>as</b> limits do not <b>apply</b> <b>to</b> this transaction fee
        <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">TransactionFee::pay_fee</a>(<a href="Libra.md#0x1_Libra_withdraw">Libra::withdraw</a>(coin, transaction_fee_amount))
    }
}
</code></pre>



</details>

<a name="0x1_LibraAccount_writeset_epilogue"></a>

## Function `writeset_epilogue`

Epilogue for WriteSet trasnaction


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_writeset_epilogue">writeset_epilogue</a>(lr_account: &signer, writeset_payload: vector&lt;u8&gt;, txn_sequence_number: u64, should_trigger_reconfiguration: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_writeset_epilogue">writeset_epilogue</a>(
    lr_account: &signer,
    writeset_payload: vector&lt;u8&gt;,
    txn_sequence_number: u64,
    should_trigger_reconfiguration: bool,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a>, <a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>, <a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a> {
    <b>let</b> writeset_events_ref = borrow_global_mut&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_UpgradeEvent">UpgradeEvent</a>&gt;(
        &<b>mut</b> writeset_events_ref.upgrade_events,
        <a href="LibraAccount.md#0x1_LibraAccount_UpgradeEvent">UpgradeEvent</a> { writeset_payload },
    );
    // Currency code don't matter here <b>as</b> it won't be charged anyway.
    <a href="LibraAccount.md#0x1_LibraAccount_epilogue">epilogue</a>&lt;<a href="LBR.md#0x1_LBR_LBR">LBR::LBR</a>&gt;(lr_account, txn_sequence_number, 0, 0, 0);
    <b>if</b> (should_trigger_reconfiguration) <a href="LibraConfig.md#0x1_LibraConfig_reconfigure">LibraConfig::reconfigure</a>(lr_account)
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_validator_account"></a>

## Function `create_validator_account`



<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_validator_account">create_validator_account</a>(lr_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_validator_account">create_validator_account</a>(
    lr_account: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> new_account = <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    // The lr_account account is verified <b>to</b> have the libra root role in `<a href="Roles.md#0x1_Roles_new_validator_role">Roles::new_validator_role</a>`
    <a href="Roles.md#0x1_Roles_new_validator_role">Roles::new_validator_role</a>(lr_account, &new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="ValidatorConfig.md#0x1_ValidatorConfig_publish">ValidatorConfig::publish</a>(&new_account, lr_account, human_name);
    <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateValidatorAccountAbortsIf">CreateValidatorAccountAbortsIf</a>;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateValidatorAccountEnsures">CreateValidatorAccountEnsures</a>;
</code></pre>




<a name="0x1_LibraAccount_CreateValidatorAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateValidatorAccountAbortsIf">CreateValidatorAccountAbortsIf</a> {
    lr_account: signer;
    new_account_address: address;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_MakeAccountAbortsIf">MakeAccountAbortsIf</a>{addr: new_account_address};
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>aborts_if</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">ValidatorConfig::exists_config</a>(new_account_address) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
}
</code></pre>




<a name="0x1_LibraAccount_CreateValidatorAccountEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateValidatorAccountEnsures">CreateValidatorAccountEnsures</a> {
    new_account_address: address;
    <b>include</b> <a href="Roles.md#0x1_Roles_GrantRole">Roles::GrantRole</a>{addr: new_account_address, role_id: <a href="Roles.md#0x1_Roles_VALIDATOR_ROLE_ID">Roles::VALIDATOR_ROLE_ID</a>};
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(new_account_address);
    <b>ensures</b> <a href="ValidatorConfig.md#0x1_ValidatorConfig_exists_config">ValidatorConfig::exists_config</a>(new_account_address);
}
</code></pre>



</details>

<a name="0x1_LibraAccount_create_validator_operator_account"></a>

## Function `create_validator_operator_account`



<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_validator_operator_account">create_validator_operator_account</a>(lr_account: &signer, new_account_address: address, auth_key_prefix: vector&lt;u8&gt;, human_name: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraAccount.md#0x1_LibraAccount_create_validator_operator_account">create_validator_operator_account</a>(
    lr_account: &signer,
    new_account_address: address,
    auth_key_prefix: vector&lt;u8&gt;,
    human_name: vector&lt;u8&gt;,
) <b>acquires</b> <a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a> {
    <b>let</b> new_account = <a href="LibraAccount.md#0x1_LibraAccount_create_signer">create_signer</a>(new_account_address);
    // The lr_account is verified <b>to</b> have the libra root role in `<a href="Roles.md#0x1_Roles_new_validator_operator_role">Roles::new_validator_operator_role</a>`
    <a href="Roles.md#0x1_Roles_new_validator_operator_role">Roles::new_validator_operator_role</a>(lr_account, &new_account);
    <a href="Event.md#0x1_Event_publish_generator">Event::publish_generator</a>(&new_account);
    <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_publish">ValidatorOperatorConfig::publish</a>(&new_account, lr_account, human_name);
    <a href="LibraAccount.md#0x1_LibraAccount_make_account">make_account</a>(new_account, auth_key_prefix)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateValidatorOperatorAccountAbortsIf">CreateValidatorOperatorAccountAbortsIf</a>;
<b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateValidatorOperatorAccountEnsures">CreateValidatorOperatorAccountEnsures</a>;
</code></pre>




<a name="0x1_LibraAccount_CreateValidatorOperatorAccountAbortsIf"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateValidatorOperatorAccountAbortsIf">CreateValidatorOperatorAccountAbortsIf</a> {
    lr_account: signer;
    new_account_address: address;
    <b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
    <b>include</b> <a href="LibraAccount.md#0x1_LibraAccount_MakeAccountAbortsIf">MakeAccountAbortsIf</a>{addr: new_account_address};
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
    <b>aborts_if</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">ValidatorOperatorConfig::has_validator_operator_config</a>(new_account_address) <b>with</b> <a href="Errors.md#0x1_Errors_ALREADY_PUBLISHED">Errors::ALREADY_PUBLISHED</a>;
}
</code></pre>




<a name="0x1_LibraAccount_CreateValidatorOperatorAccountEnsures"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_CreateValidatorOperatorAccountEnsures">CreateValidatorOperatorAccountEnsures</a> {
    new_account_address: address;
    <b>include</b> <a href="Roles.md#0x1_Roles_GrantRole">Roles::GrantRole</a>{addr: new_account_address, role_id: <a href="Roles.md#0x1_Roles_VALIDATOR_OPERATOR_ROLE_ID">Roles::VALIDATOR_OPERATOR_ROLE_ID</a>};
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(new_account_address);
    <b>ensures</b> <a href="ValidatorOperatorConfig.md#0x1_ValidatorOperatorConfig_has_validator_operator_config">ValidatorOperatorConfig::has_validator_operator_config</a>(new_account_address);
}
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



Returns field <code>key_rotation_capability</code> of the LibraAccount under <code>addr</code>.


<a name="0x1_LibraAccount_spec_get_key_rotation_cap_field"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap_field">spec_get_key_rotation_cap_field</a>(addr: address): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a>&gt; {
    <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr).key_rotation_capability
}
</code></pre>


Returns the KeyRotationCapability of the field <code>key_rotation_capability</code>.


<a name="0x1_LibraAccount_spec_get_key_rotation_cap"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">spec_get_key_rotation_cap</a>(addr: address): <a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a> {
    <a href="Option.md#0x1_Option_spec_get">Option::spec_get</a>(<a href="LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap_field">spec_get_key_rotation_cap_field</a>(addr))
}
<a name="0x1_LibraAccount_spec_has_key_rotation_cap"></a>
<b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_has_key_rotation_cap">spec_has_key_rotation_cap</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<a href="LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap_field">spec_get_key_rotation_cap_field</a>(addr))
}
</code></pre>


Returns true if the LibraAccount at <code>addr</code> holds
<code><a href="LibraAccount.md#0x1_LibraAccount_KeyRotationCapability">KeyRotationCapability</a></code> for itself.


<a name="0x1_LibraAccount_spec_holds_own_key_rotation_cap"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_key_rotation_cap">spec_holds_own_key_rotation_cap</a>(addr: address): bool {
    <a href="LibraAccount.md#0x1_LibraAccount_spec_has_key_rotation_cap">spec_has_key_rotation_cap</a>(addr)
    && addr == <a href="LibraAccount.md#0x1_LibraAccount_spec_get_key_rotation_cap">spec_get_key_rotation_cap</a>(addr).account_address
}
</code></pre>


Returns true if <code><a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a></code> is published.


<a name="0x1_LibraAccount_spec_has_account_operations_cap"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_has_account_operations_cap">spec_has_account_operations_cap</a>(): bool {
    <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>


Returns field <code>withdrawal_capability</code> of LibraAccount under <code>addr</code>.


<a name="0x1_LibraAccount_spec_get_withdraw_cap_field"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap_field">spec_get_withdraw_cap_field</a>(addr: address): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a>&gt; {
    <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr).withdrawal_capability
}
</code></pre>


Returns the WithdrawCapability of the field <code>withdrawal_capability</code>.


<a name="0x1_LibraAccount_spec_get_withdraw_cap"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">spec_get_withdraw_cap</a>(addr: address): <a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a> {
    <a href="Option.md#0x1_Option_spec_get">Option::spec_get</a>(<a href="LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap_field">spec_get_withdraw_cap_field</a>(addr))
}
</code></pre>


Returns true if the LibraAccount at <code>addr</code> holds a <code><a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code>.


<a name="0x1_LibraAccount_spec_has_withdraw_cap"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_has_withdraw_cap">spec_has_withdraw_cap</a>(addr: address): bool {
    <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<a href="LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap_field">spec_get_withdraw_cap_field</a>(addr))
}
</code></pre>


Returns true if the LibraAccount at <code>addr</code> holds <code><a href="LibraAccount.md#0x1_LibraAccount_WithdrawCapability">WithdrawCapability</a></code> for itself.


<a name="0x1_LibraAccount_spec_holds_own_withdraw_cap"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_withdraw_cap">spec_holds_own_withdraw_cap</a>(addr: address): bool {
    <a href="LibraAccount.md#0x1_LibraAccount_spec_has_withdraw_cap">spec_has_withdraw_cap</a>(addr)
    && addr == <a href="LibraAccount.md#0x1_LibraAccount_spec_get_withdraw_cap">spec_get_withdraw_cap</a>(addr).account_address
}
</code></pre>


Returns the value of the coins held in the specified currency.


<a name="0x1_LibraAccount_spec_get_balance_value"></a>


<pre><code><b>define</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_get_balance_value">spec_get_balance_value</a>&lt;Currency&gt;(addr: address): u64 {
    <a href="Libra.md#0x1_Libra_value">Libra::value</a>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Currency&gt;&gt;(addr).coin)
}
</code></pre>




<a name="0x1_LibraAccount_EnsuresHasKeyRotationCap"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_EnsuresHasKeyRotationCap">EnsuresHasKeyRotationCap</a> {
    account: signer;
    <a name="0x1_LibraAccount_addr$79"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_key_rotation_cap">spec_holds_own_key_rotation_cap</a>(addr);
}
</code></pre>




<a name="0x1_LibraAccount_PreserveKeyRotationCapAbsence"></a>

The absence of KeyRotationCap is preserved.


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_PreserveKeyRotationCapAbsence">PreserveKeyRotationCapAbsence</a> {
    <b>ensures</b> <b>forall</b> addr1: address:
        <b>old</b>(!<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1) || !<a href="LibraAccount.md#0x1_LibraAccount_spec_has_key_rotation_cap">spec_has_key_rotation_cap</a>(addr1)) ==&gt;
            (!<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1) || !<a href="LibraAccount.md#0x1_LibraAccount_spec_has_key_rotation_cap">spec_has_key_rotation_cap</a>(addr1));
}
</code></pre>



the permission "RotateAuthenticationKey(addr)" is granted to the account at addr [H16].
When an account is created, its KeyRotationCapability is granted to the account.


<pre><code><b>apply</b> <a href="LibraAccount.md#0x1_LibraAccount_EnsuresHasKeyRotationCap">EnsuresHasKeyRotationCap</a>{account: new_account} <b>to</b> make_account;
</code></pre>


Only <code>make_account</code> creates KeyRotationCap [H16][I16]. <code>create_*_account</code> only calls
<code>make_account</code>, and does not pack KeyRotationCap by itself.
<code>restore_key_rotation_capability</code> restores KeyRotationCap, and does not create new one.


<pre><code><b>apply</b> <a href="LibraAccount.md#0x1_LibraAccount_PreserveKeyRotationCapAbsence">PreserveKeyRotationCapAbsence</a> <b>to</b> * <b>except</b> make_account, create_*_account,
      restore_key_rotation_capability, initialize;
</code></pre>


Every account holds either no key rotation capability (because KeyRotationCapability has been delegated)
or the key rotation capability for addr itself [H16].


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> addr1: address <b>where</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1):
    <a href="LibraAccount.md#0x1_LibraAccount_delegated_key_rotation_capability">delegated_key_rotation_capability</a>(addr1) || <a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_key_rotation_cap">spec_holds_own_key_rotation_cap</a>(addr1);
</code></pre>




<a name="0x1_LibraAccount_EnsuresWithdrawalCap"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_EnsuresWithdrawalCap">EnsuresWithdrawalCap</a> {
    account: signer;
    <a name="0x1_LibraAccount_addr$80"></a>
    <b>let</b> addr = <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account);
    <b>ensures</b> <a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_withdraw_cap">spec_holds_own_withdraw_cap</a>(addr);
}
</code></pre>




<a name="0x1_LibraAccount_PreserveWithdrawCapAbsence"></a>

The absence of WithdrawCap is preserved.


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_PreserveWithdrawCapAbsence">PreserveWithdrawCapAbsence</a> {
    <b>ensures</b> <b>forall</b> addr1: address:
        <b>old</b>(!<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1) || <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1).withdrawal_capability)) ==&gt;
            (!<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1) || <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1).withdrawal_capability));
}
</code></pre>



the permission "WithdrawalCapability(addr)" is granted to the account at addr [H17].
When an account is created, its WithdrawCapability is granted to the account.


<pre><code><b>apply</b> <a href="LibraAccount.md#0x1_LibraAccount_EnsuresWithdrawalCap">EnsuresWithdrawalCap</a>{account: new_account} <b>to</b> make_account;
</code></pre>


Only <code>make_account</code> creates WithdrawCap [H17][I17]. <code>create_*_account</code> only calls
<code>make_account</code>, and does not pack KeyRotationCap by itself.
<code>restore_withdraw_capability</code> restores WithdrawCap, and does not create new one.


<pre><code><b>apply</b> <a href="LibraAccount.md#0x1_LibraAccount_PreserveWithdrawCapAbsence">PreserveWithdrawCapAbsence</a> <b>to</b> * <b>except</b> make_account, create_*_account,
        restore_withdraw_capability, initialize;
</code></pre>


Every account holds either no withdraw capability (because withdraw cap has been delegated)
or the withdraw capability for addr itself [H17].


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> addr1: address <b>where</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1):
    <a href="LibraAccount.md#0x1_LibraAccount_delegated_withdraw_capability">delegated_withdraw_capability</a>(addr1) || <a href="LibraAccount.md#0x1_LibraAccount_spec_holds_own_withdraw_cap">spec_holds_own_withdraw_cap</a>(addr1);
</code></pre>


Invariants

Every address that has a published RoleId also has a published Account.


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> addr1: address <b>where</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1): <b>exists</b>&lt;<a href="Roles.md#0x1_Roles_RoleId">Roles::RoleId</a>&gt;(addr1);
</code></pre>


Accounts are never deleted.


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>] <b>forall</b> addr1: address <b>where</b> <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1)): <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1);
</code></pre>


After genesis, the <code><a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a></code> exists.


<pre><code><b>invariant</b> [<b>global</b>]
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_AccountOperationsCapability">AccountOperationsCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


After genesis, the <code><a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a></code> exists.


<pre><code><b>invariant</b> [<b>global</b>]
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_LibraWriteSetManager">LibraWriteSetManager</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


Every address that has a published account has a published RoleId


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> addr1: address <b>where</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1): <b>exists</b>&lt;<a href="Roles.md#0x1_Roles_RoleId">Roles::RoleId</a>&gt;(addr1);
</code></pre>


Every address that has a published account has a published FreezingBit


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> addr1: address <b>where</b> <a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1): <b>exists</b>&lt;<a href="AccountFreezing.md#0x1_AccountFreezing_FreezingBit">AccountFreezing::FreezingBit</a>&gt;(addr1);
</code></pre>


only rotate_authentication_key can rotate authentication_key [H16].


<a name="0x1_LibraAccount_AuthenticationKeyRemainsSame"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_AuthenticationKeyRemainsSame">AuthenticationKeyRemainsSame</a> {
    <b>ensures</b> <b>forall</b> addr1: address <b>where</b> <b>old</b>(<a href="LibraAccount.md#0x1_LibraAccount_exists_at">exists_at</a>(addr1)):
        <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1).authentication_key == <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount">LibraAccount</a>&gt;(addr1).authentication_key);
}
</code></pre>




<pre><code><b>apply</b> <a href="LibraAccount.md#0x1_LibraAccount_AuthenticationKeyRemainsSame">AuthenticationKeyRemainsSame</a> <b>to</b> *, *&lt;T&gt; <b>except</b> rotate_authentication_key;
</code></pre>


If an account has a balance, the role of the account is compatible with having a balance.
ref: Only reasonable accounts have currencies.


<pre><code><b>invariant</b> [<b>global</b>] <b>forall</b> t: type: <b>forall</b> addr1: address <b>where</b> <b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;t&gt;&gt;(addr1):
    <a href="Roles.md#0x1_Roles_spec_can_hold_balance_addr">Roles::spec_can_hold_balance_addr</a>(addr1);
</code></pre>


only withdraw_from and its helper and clients can withdraw [H17].


<a name="0x1_LibraAccount_BalanceNotDecrease"></a>


<pre><code><b>schema</b> <a href="LibraAccount.md#0x1_LibraAccount_BalanceNotDecrease">BalanceNotDecrease</a>&lt;Token&gt; {
    <b>ensures</b> <b>forall</b> addr1: address <b>where</b> <b>old</b>(<b>exists</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr1)):
        <b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr1).coin.value &gt;= <b>old</b>(<b>global</b>&lt;<a href="LibraAccount.md#0x1_LibraAccount_Balance">Balance</a>&lt;Token&gt;&gt;(addr1).coin.value);
}
</code></pre>




<pre><code><b>apply</b> <a href="LibraAccount.md#0x1_LibraAccount_BalanceNotDecrease">BalanceNotDecrease</a>&lt;Token&gt; <b>to</b> *&lt;Token&gt;
    <b>except</b> withdraw_from, withdraw_from_balance, staple_lbr, unstaple_lbr,
        preburn, pay_from, epilogue, failure_epilogue, success_epilogue;
</code></pre>

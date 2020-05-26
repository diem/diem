
<a name="0x0_TransactionFee"></a>

# Module `0x0::TransactionFee`

### Table of Contents

-  [Struct `TransactionFees`](#0x0_TransactionFee_TransactionFees)
-  [Function `initialize_transaction_fees`](#0x0_TransactionFee_initialize_transaction_fees)
-  [Function `distribute_transaction_fees`](#0x0_TransactionFee_distribute_transaction_fees)
-  [Function `distribute_transaction_fees_internal`](#0x0_TransactionFee_distribute_transaction_fees_internal)
-  [Function `per_validator_distribution_amount`](#0x0_TransactionFee_per_validator_distribution_amount)



<a name="0x0_TransactionFee_TransactionFees"></a>

## Struct `TransactionFees`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_TransactionFee_TransactionFees">TransactionFees</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>fee_withdrawal_capability: <a href="libra_account.md#0x0_LibraAccount_WithdrawalCapability">LibraAccount::WithdrawalCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_TransactionFee_initialize_transaction_fees"></a>

## Function `initialize_transaction_fees`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_initialize_transaction_fees">initialize_transaction_fees</a>(fee_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_initialize_transaction_fees">initialize_transaction_fees</a>(fee_account: &signer) {
    Transaction::assert(<a href="signer.md#0x0_Signer_address_of">Signer::address_of</a>(fee_account) == 0xFEE, 0);
    move_to(
        fee_account,
        <a href="#0x0_TransactionFee_TransactionFees">TransactionFees</a> {
            fee_withdrawal_capability: <a href="libra_account.md#0x0_LibraAccount_extract_sender_withdrawal_capability">LibraAccount::extract_sender_withdrawal_capability</a>(),
        }
    );
}
</code></pre>



</details>

<a name="0x0_TransactionFee_distribute_transaction_fees"></a>

## Function `distribute_transaction_fees`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;Token&gt;()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_TransactionFee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;Token&gt;() <b>acquires</b> <a href="#0x0_TransactionFee_TransactionFees">TransactionFees</a> {
  // Can only be invoked by LibraVM privilege.
  Transaction::assert(Transaction::sender() == 0x0, 33);

  <b>let</b> num_validators = <a href="libra_system.md#0x0_LibraSystem_validator_set_size">LibraSystem::validator_set_size</a>();
  <b>let</b> amount_collected = <a href="libra_account.md#0x0_LibraAccount_balance">LibraAccount::balance</a>&lt;Token&gt;(0xFEE);

  // If amount_collected == 0, this will also <b>return</b> early
  <b>if</b> (amount_collected &lt; num_validators) <b>return</b> ();

  // Calculate the amount of money <b>to</b> be dispursed, along with the remainder.
  <b>let</b> amount_to_distribute_per_validator = <a href="#0x0_TransactionFee_per_validator_distribution_amount">per_validator_distribution_amount</a>(
      amount_collected,
      num_validators
  );

  // Iterate through the validators distributing fees equally
  <a href="#0x0_TransactionFee_distribute_transaction_fees_internal">distribute_transaction_fees_internal</a>&lt;Token&gt;(
      amount_to_distribute_per_validator,
      num_validators,
  );
}
</code></pre>



</details>

<a name="0x0_TransactionFee_distribute_transaction_fees_internal"></a>

## Function `distribute_transaction_fees_internal`



<pre><code><b>fun</b> <a href="#0x0_TransactionFee_distribute_transaction_fees_internal">distribute_transaction_fees_internal</a>&lt;Token&gt;(amount_to_distribute_per_validator: u64, num_validators: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_TransactionFee_distribute_transaction_fees_internal">distribute_transaction_fees_internal</a>&lt;Token&gt;(
    amount_to_distribute_per_validator: u64,
    num_validators: u64,
) <b>acquires</b> <a href="#0x0_TransactionFee_TransactionFees">TransactionFees</a> {
    <b>let</b> distribution_resource = borrow_global&lt;<a href="#0x0_TransactionFee_TransactionFees">TransactionFees</a>&gt;(0xFEE);
    <b>let</b> index = 0;

    <b>while</b> (index &lt; num_validators) {

        <b>let</b> addr = <a href="libra_system.md#0x0_LibraSystem_get_ith_validator_address">LibraSystem::get_ith_validator_address</a>(index);
        // Increment the index into the validator set.
        index = index + 1;

        <a href="libra_account.md#0x0_LibraAccount_pay_from_capability">LibraAccount::pay_from_capability</a>&lt;Token&gt;(
            addr,
            &distribution_resource.fee_withdrawal_capability,
            amount_to_distribute_per_validator,
            x"",
            x""
        );
       }
}
</code></pre>



</details>

<a name="0x0_TransactionFee_per_validator_distribution_amount"></a>

## Function `per_validator_distribution_amount`



<pre><code><b>fun</b> <a href="#0x0_TransactionFee_per_validator_distribution_amount">per_validator_distribution_amount</a>(amount_collected: u64, num_validators: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_TransactionFee_per_validator_distribution_amount">per_validator_distribution_amount</a>(amount_collected: u64, num_validators: u64): u64 {
    Transaction::assert(num_validators != 0, 0);
    <b>let</b> validator_payout = amount_collected / num_validators;
    validator_payout
}
</code></pre>



</details>

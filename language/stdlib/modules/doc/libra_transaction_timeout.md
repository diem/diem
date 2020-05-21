
<a name="0x0_LibraTransactionTimeout"></a>

# Module `0x0::LibraTransactionTimeout`

### Table of Contents

-  [Struct `TTL`](#0x0_LibraTransactionTimeout_TTL)
-  [Function `initialize`](#0x0_LibraTransactionTimeout_initialize)
-  [Function `set_timeout`](#0x0_LibraTransactionTimeout_set_timeout)
-  [Function `is_valid_transaction_timestamp`](#0x0_LibraTransactionTimeout_is_valid_transaction_timestamp)



<a name="0x0_LibraTransactionTimeout_TTL"></a>

## Struct `TTL`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraTransactionTimeout_TTL">TTL</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>duration_microseconds: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraTransactionTimeout_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTransactionTimeout_initialize">initialize</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTransactionTimeout_initialize">initialize</a>() {

  // Only callable by the <a href="association.md#0x0_Association">Association</a> address
  Transaction::assert(Transaction::sender() == 0xA550C18, 1);
  // Currently set <b>to</b> 1day.
  move_to_sender&lt;<a href="#0x0_LibraTransactionTimeout_TTL">TTL</a>&gt;(<a href="#0x0_LibraTransactionTimeout_TTL">TTL</a> {duration_microseconds: 86400000000});
}
</code></pre>



</details>

<a name="0x0_LibraTransactionTimeout_set_timeout"></a>

## Function `set_timeout`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTransactionTimeout_set_timeout">set_timeout</a>(new_duration: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTransactionTimeout_set_timeout">set_timeout</a>(new_duration: u64) <b>acquires</b> <a href="#0x0_LibraTransactionTimeout_TTL">TTL</a> {
  // Only callable by the <a href="association.md#0x0_Association">Association</a> address
  Transaction::assert(Transaction::sender() == 0xA550C18, 1);

  <b>let</b> timeout = borrow_global_mut&lt;<a href="#0x0_LibraTransactionTimeout_TTL">TTL</a>&gt;(0xA550C18);
  timeout.duration_microseconds = new_duration;
}
</code></pre>



</details>

<a name="0x0_LibraTransactionTimeout_is_valid_transaction_timestamp"></a>

## Function `is_valid_transaction_timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(timestamp: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraTransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(timestamp: u64): bool <b>acquires</b> <a href="#0x0_LibraTransactionTimeout_TTL">TTL</a> {
  // Reject timestamp greater than u64::MAX / 1_000_000;
  <b>if</b>(timestamp &gt; 9223372036854) {
    <b>return</b> <b>false</b>
  };

  <b>let</b> current_block_time = <a href="libra_time.md#0x0_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
  <b>let</b> timeout = borrow_global&lt;<a href="#0x0_LibraTransactionTimeout_TTL">TTL</a>&gt;(0xA550C18).duration_microseconds;
  <b>let</b> _max_txn_time = current_block_time + timeout;

  <b>let</b> txn_time_microseconds = timestamp * 1000000;
  // TODO: Add LibraTimestamp::is_before_exclusive(&txn_time_microseconds, &max_txn_time)
  //       This is causing flaky test right now. The reason is that we will <b>use</b> this logic for AC, where its wall
  //       clock time might be out of sync with the real block time stored in StateStore.
  //       See details in issue #2346.
  current_block_time &lt; txn_time_microseconds
}
</code></pre>



</details>

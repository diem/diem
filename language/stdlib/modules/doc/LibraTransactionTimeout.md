
<a name="0x1_LibraTransactionTimeout"></a>

# Module `0x1::LibraTransactionTimeout`

### Table of Contents

-  [Resource `TTL`](#0x1_LibraTransactionTimeout_TTL)
-  [Const `MAX_TIMESTAMP`](#0x1_LibraTransactionTimeout_MAX_TIMESTAMP)
-  [Const `MICROS_MULTIPLIER`](#0x1_LibraTransactionTimeout_MICROS_MULTIPLIER)
-  [Const `ONE_DAY_MICROS`](#0x1_LibraTransactionTimeout_ONE_DAY_MICROS)
-  [Const `ETTL`](#0x1_LibraTransactionTimeout_ETTL)
-  [Function `initialize`](#0x1_LibraTransactionTimeout_initialize)
-  [Function `is_initialized`](#0x1_LibraTransactionTimeout_is_initialized)
-  [Function `set_timeout`](#0x1_LibraTransactionTimeout_set_timeout)
-  [Function `is_valid_transaction_timestamp`](#0x1_LibraTransactionTimeout_is_valid_transaction_timestamp)
-  [Specification](#0x1_LibraTransactionTimeout_Specification)



<a name="0x1_LibraTransactionTimeout_TTL"></a>

## Resource `TTL`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraTransactionTimeout_TTL">TTL</a>
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

<a name="0x1_LibraTransactionTimeout_MAX_TIMESTAMP"></a>

## Const `MAX_TIMESTAMP`



<pre><code><b>const</b> MAX_TIMESTAMP: u64 = 18446744073709;
</code></pre>



<a name="0x1_LibraTransactionTimeout_MICROS_MULTIPLIER"></a>

## Const `MICROS_MULTIPLIER`



<pre><code><b>const</b> MICROS_MULTIPLIER: u64 = 1000000;
</code></pre>



<a name="0x1_LibraTransactionTimeout_ONE_DAY_MICROS"></a>

## Const `ONE_DAY_MICROS`



<pre><code><b>const</b> ONE_DAY_MICROS: u64 = 86400000000;
</code></pre>



<a name="0x1_LibraTransactionTimeout_ETTL"></a>

## Const `ETTL`

The
<code><a href="#0x1_LibraTransactionTimeout_TTL">TTL</a></code> resource was not in the required state


<pre><code><b>const</b> ETTL: u64 = 0;
</code></pre>



<a name="0x1_LibraTransactionTimeout_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionTimeout_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionTimeout_initialize">initialize</a>(lr_account: &signer) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    // Operational constraint, only callable by the libra root account
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);
    <b>assert</b>(!<a href="#0x1_LibraTransactionTimeout_is_initialized">is_initialized</a>(), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(ETTL));
    // Currently set <b>to</b> 1day.
    move_to(lr_account, <a href="#0x1_LibraTransactionTimeout_TTL">TTL</a> {duration_microseconds: ONE_DAY_MICROS});
}
</code></pre>



</details>

<a name="0x1_LibraTransactionTimeout_is_initialized"></a>

## Function `is_initialized`



<pre><code><b>fun</b> <a href="#0x1_LibraTransactionTimeout_is_initialized">is_initialized</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraTransactionTimeout_is_initialized">is_initialized</a>(): bool {
    exists&lt;<a href="#0x1_LibraTransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_LibraTransactionTimeout_set_timeout"></a>

## Function `set_timeout`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionTimeout_set_timeout">set_timeout</a>(lr_account: &signer, new_duration: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionTimeout_set_timeout">set_timeout</a>(lr_account: &signer, new_duration: u64) <b>acquires</b> <a href="#0x1_LibraTransactionTimeout_TTL">TTL</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <b>let</b> timeout = borrow_global_mut&lt;<a href="#0x1_LibraTransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    timeout.duration_microseconds = new_duration;
}
</code></pre>



</details>

<a name="0x1_LibraTransactionTimeout_is_valid_transaction_timestamp"></a>

## Function `is_valid_transaction_timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(timestamp: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(timestamp: u64): bool <b>acquires</b> <a href="#0x1_LibraTransactionTimeout_TTL">TTL</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    // Reject timestamp greater than u64::MAX / 1_000_000.
    // This allows converting the timestamp from seconds <b>to</b> microseconds.
    <b>if</b> (timestamp &gt; MAX_TIMESTAMP) {
      <b>return</b> <b>false</b>
    };

    <b>let</b> current_block_time = <a href="LibraTimestamp.md#0x1_LibraTimestamp_now_microseconds">LibraTimestamp::now_microseconds</a>();
    <b>let</b> timeout = borrow_global&lt;<a href="#0x1_LibraTransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).duration_microseconds;
    <b>let</b> _max_txn_time = current_block_time + timeout;

    <b>let</b> txn_time_microseconds = timestamp * MICROS_MULTIPLIER;
    // TODO: Add LibraTimestamp::is_before_exclusive(&txn_time_microseconds, &max_txn_time)
    //       This is causing flaky test right now. The reason is that we will <b>use</b> this logic for AC, where its wall
    //       clock time might be out of sync with the real block time stored in StateStore.
    //       See details in issue #2346.
    current_block_time &lt; txn_time_microseconds
}
</code></pre>



</details>

<a name="0x1_LibraTransactionTimeout_Specification"></a>

## Specification



<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="#0x1_LibraTransactionTimeout_is_initialized">is_initialized</a>();
</code></pre>

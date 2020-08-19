
<a name="0x1_LibraTimestamp"></a>

# Module `0x1::LibraTimestamp`

### Table of Contents

-  [Resource `CurrentTimeMicroseconds`](#0x1_LibraTimestamp_CurrentTimeMicroseconds)
-  [Resource `TimeHasStarted`](#0x1_LibraTimestamp_TimeHasStarted)
-  [Const `MICRO_CONVERSION_FACTOR`](#0x1_LibraTimestamp_MICRO_CONVERSION_FACTOR)
-  [Const `ENOT_GENESIS`](#0x1_LibraTimestamp_ENOT_GENESIS)
-  [Const `ENOT_OPERATING`](#0x1_LibraTimestamp_ENOT_OPERATING)
-  [Const `ETIMER_RESOURCE`](#0x1_LibraTimestamp_ETIMER_RESOURCE)
-  [Const `ETIMESTAMP`](#0x1_LibraTimestamp_ETIMESTAMP)
-  [Function `initialize`](#0x1_LibraTimestamp_initialize)
-  [Function `set_time_has_started`](#0x1_LibraTimestamp_set_time_has_started)
-  [Function `reset_time_has_started_for_test`](#0x1_LibraTimestamp_reset_time_has_started_for_test)
-  [Function `update_global_time`](#0x1_LibraTimestamp_update_global_time)
-  [Function `now_microseconds`](#0x1_LibraTimestamp_now_microseconds)
-  [Function `now_seconds`](#0x1_LibraTimestamp_now_seconds)
-  [Function `is_genesis`](#0x1_LibraTimestamp_is_genesis)
-  [Function `assert_genesis`](#0x1_LibraTimestamp_assert_genesis)
-  [Function `is_operating`](#0x1_LibraTimestamp_is_operating)
-  [Function `assert_operating`](#0x1_LibraTimestamp_assert_operating)
-  [Function `is_not_initialized`](#0x1_LibraTimestamp_is_not_initialized)
-  [Specification](#0x1_LibraTimestamp_Specification)
    -  [Function `initialize`](#0x1_LibraTimestamp_Specification_initialize)
    -  [Function `set_time_has_started`](#0x1_LibraTimestamp_Specification_set_time_has_started)
    -  [Function `update_global_time`](#0x1_LibraTimestamp_Specification_update_global_time)
    -  [Function `now_microseconds`](#0x1_LibraTimestamp_Specification_now_microseconds)
    -  [Function `now_seconds`](#0x1_LibraTimestamp_Specification_now_seconds)
    -  [Function `assert_genesis`](#0x1_LibraTimestamp_Specification_assert_genesis)
    -  [Function `assert_operating`](#0x1_LibraTimestamp_Specification_assert_operating)

This module keeps a global wall clock that stores the current Unix time in microseconds.
It interacts with the other modules in the following ways:

* Genesis: to initialize the timestamp
* VASP: to keep track of when credentials expire
* LibraSystem, LibraAccount, LibraConfig: to check if the current state is in the genesis state
* LibraBlock: to reach consensus on the global wall clock time
* AccountLimits: to limit the time of account limits


<a name="0x1_LibraTimestamp_CurrentTimeMicroseconds"></a>

## Resource `CurrentTimeMicroseconds`

A singleton resource holding the current Unix time in microseconds


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>microseconds: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraTimestamp_TimeHasStarted"></a>

## Resource `TimeHasStarted`

A singleton resource used to determine whether time has started. This
is called at the end of genesis.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>
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

<a name="0x1_LibraTimestamp_MICRO_CONVERSION_FACTOR"></a>

## Const `MICRO_CONVERSION_FACTOR`

Conversion factor between seconds and microseconds


<pre><code><b>const</b> MICRO_CONVERSION_FACTOR: u64 = 1000000;
</code></pre>



<a name="0x1_LibraTimestamp_ENOT_GENESIS"></a>

## Const `ENOT_GENESIS`

The blockchain is not in the genesis state anymore


<pre><code><b>const</b> ENOT_GENESIS: u64 = 0;
</code></pre>



<a name="0x1_LibraTimestamp_ENOT_OPERATING"></a>

## Const `ENOT_OPERATING`

The blockchain is not in an operating state yet


<pre><code><b>const</b> ENOT_OPERATING: u64 = 1;
</code></pre>



<a name="0x1_LibraTimestamp_ETIMER_RESOURCE"></a>

## Const `ETIMER_RESOURCE`

The
<code><a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a></code> resource was not in the required state


<pre><code><b>const</b> ETIMER_RESOURCE: u64 = 2;
</code></pre>



<a name="0x1_LibraTimestamp_ETIMESTAMP"></a>

## Const `ETIMESTAMP`

An invalid timestamp was provided


<pre><code><b>const</b> ETIMESTAMP: u64 = 3;
</code></pre>



<a name="0x1_LibraTimestamp_initialize"></a>

## Function `initialize`

Initializes the global wall clock time resource. This can only be called from genesis and with the
root account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(lr_account: &signer) {
    // Only callable from genesis.
    <a href="#0x1_LibraTimestamp_assert_genesis">assert_genesis</a>();

    // Only callable by libra root account
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);

    // TODO: Should the initialized value be passed in <b>to</b> genesis?
    <b>let</b> timer = <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> { microseconds: 0 };
    <b>assert</b>(!exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(ETIMER_RESOURCE));
    move_to(lr_account, timer);
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_set_time_has_started"></a>

## Function `set_time_has_started`

Marks that time has started and genesis has finished. This can only be called from genesis and with the root
account.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_set_time_has_started">set_time_has_started</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_set_time_has_started">set_time_has_started</a>(lr_account: &signer) <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    <a href="#0x1_LibraTimestamp_assert_genesis">assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(lr_account);

    // Current time must have been initialized.
    <b>assert</b>(
        exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) && <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>() == 0,
        <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(ETIMER_RESOURCE)
    );
    move_to(lr_account, <a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>{});
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_reset_time_has_started_for_test"></a>

## Function `reset_time_has_started_for_test`

Helper functions for tests to reset the time-has-started, and pretend to be in genesis.
> TODO(wrwg): we MUST have a capability which only tests can have to be able to call
> this function before we go to production.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_reset_time_has_started_for_test">reset_time_has_started_for_test</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_reset_time_has_started_for_test">reset_time_has_started_for_test</a>() <b>acquires</b> <a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a> {
    <b>if</b> (exists&lt;<a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())) {
        // Only do this <b>if</b> it actually exists.
        <b>let</b> <a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>{} = move_from&lt;<a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    }
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_update_global_time"></a>

## Function `update_global_time`

Updates the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_update_global_time">update_global_time</a>(account: &signer, proposer: address, timestamp: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_update_global_time">update_global_time</a>(
    account: &signer,
    proposer: address,
    timestamp: u64
) <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    <a href="#0x1_LibraTimestamp_assert_operating">assert_operating</a>();
    // Can only be invoked by LibraVM signer.
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_vm">CoreAddresses::assert_vm</a>(account);

    <b>let</b> global_timer = borrow_global_mut&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <b>let</b> now = global_timer.microseconds;
    <b>if</b> (proposer == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>()) {
        // NIL block with null address <b>as</b> proposer. Timestamp must be equal.
        <b>assert</b>(now == timestamp, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(ETIMESTAMP));
    } <b>else</b> {
        // Normal block. Time must advance
        <b>assert</b>(now &lt; timestamp, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(ETIMESTAMP));
    };
    global_timer.microseconds = timestamp;
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_now_microseconds"></a>

## Function `now_microseconds`

Gets the current time in microseconds.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64 <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    <b>assert</b>(
        exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(ETIMER_RESOURCE)
    );
    borrow_global&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).microseconds
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_now_seconds"></a>

## Function `now_seconds`

Gets the current time in seconds.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_seconds">now_seconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_seconds">now_seconds</a>(): u64 <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>() / MICRO_CONVERSION_FACTOR
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_is_genesis"></a>

## Function `is_genesis`

Helper function to determine if Libra is in genesis state.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(): bool {
    !exists&lt;<a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_assert_genesis"></a>

## Function `assert_genesis`

Helper function to assert genesis state.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_assert_genesis">assert_genesis</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_assert_genesis">assert_genesis</a>() {
    <b>assert</b>(<a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(ENOT_GENESIS));
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_is_operating"></a>

## Function `is_operating`

Helper function to determine if Libra is operating. This is the same as
<code>!<a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>()</code> and is provided
for convenience. Testing
<code><a href="#0x1_LibraTimestamp_is_operating">is_operating</a>()</code> is more frequent than
<code><a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>()</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_operating">is_operating</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_operating">is_operating</a>(): bool {
    exists&lt;<a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_assert_operating"></a>

## Function `assert_operating`

Helper function to assert operating (!genesis) state.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_assert_operating">assert_operating</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_assert_operating">assert_operating</a>() {
    <b>assert</b>(<a href="#0x1_LibraTimestamp_is_operating">is_operating</a>(), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(ENOT_OPERATING));
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_is_not_initialized"></a>

## Function `is_not_initialized`

Helper function to determine whether the CurrentTime has been not fully setup. This returns true
even if the resource exists but the time is still set to zero.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_not_initialized">is_not_initialized</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_not_initialized">is_not_initialized</a>(): bool <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
   !exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) || <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>() == 0
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_Specification"></a>

## Specification


Verify all functions in this module.


<pre><code>pragma verify;
</code></pre>


All functions which do not have an
<code><b>aborts_if</b></code> specification in this module are implicitly declared
to never abort.


<pre><code>pragma aborts_if_is_strict;
</code></pre>



After genesis, the time stamp and time-has-started marker are published.


<pre><code><b>invariant</b> [<b>global</b>]
    <a href="#0x1_LibraTimestamp_is_operating">is_operating</a>() ==&gt; exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
</code></pre>


After genesis, time progresses monotonically.


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>]
    <a href="#0x1_LibraTimestamp_is_operating">is_operating</a>() ==&gt; <b>old</b>(<a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>()) &lt;= <a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>();
</code></pre>



<a name="0x1_LibraTimestamp_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(lr_account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_LibraTimestamp_AbortsIfNotGenesis">AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>aborts_if</b> <a href="#0x1_LibraTimestamp_spec_timer_initialized">spec_timer_initialized</a>() with Errors::ALREADY_PUBLISHED;
<b>ensures</b> <a href="#0x1_LibraTimestamp_spec_timer_initialized">spec_timer_initialized</a>();
<b>ensures</b> <a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>() == 0;
</code></pre>


A helper to check whether the association root account has a CurrentTimeMicroseconds.


<a name="0x1_LibraTimestamp_spec_timer_initialized"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_spec_timer_initialized">spec_timer_initialized</a>(): bool {
exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>



<a name="0x1_LibraTimestamp_Specification_set_time_has_started"></a>

### Function `set_time_has_started`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_set_time_has_started">set_time_has_started</a>(lr_account: &signer)
</code></pre>



This function can currently not be verified standalone. Once we publish the
<code><a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a></code> resource, all invariants in the system which describe the state of the
system after genesis will be asserted. The caller need to establish a state where
these assertions verify.


<pre><code>pragma verify = <b>false</b>;
<b>include</b> <a href="#0x1_LibraTimestamp_AbortsIfNotGenesis">AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>aborts_if</b> !<a href="#0x1_LibraTimestamp_spec_timer_initialized">spec_timer_initialized</a>() || <a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>() != 0 with Errors::NOT_PUBLISHED;
<b>ensures</b> <a href="#0x1_LibraTimestamp_is_operating">is_operating</a>();
</code></pre>



<a name="0x1_LibraTimestamp_Specification_update_global_time"></a>

### Function `update_global_time`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_update_global_time">update_global_time</a>(account: &signer, proposer: address, timestamp: u64)
</code></pre>




<pre><code><b>include</b> <a href="#0x1_LibraTimestamp_AbortsIfNotOperating">AbortsIfNotOperating</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotVM">CoreAddresses::AbortsIfNotVM</a>;
<a name="0x1_LibraTimestamp_now$14"></a>
<b>let</b> now = <b>old</b>(<a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>());
<b>aborts_if</b> [<b>assume</b>]
    (<b>if</b> (proposer == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>()) {
        now != timestamp
     } <b>else</b>  {
        now &gt;= timestamp
     }
    )
    with Errors::INVALID_ARGUMENT;
<b>ensures</b> <a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>() == timestamp;
</code></pre>



<a name="0x1_LibraTimestamp_Specification_now_microseconds"></a>

### Function `now_microseconds`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_LibraTimestamp_AbortsIfNoTime">AbortsIfNoTime</a>;
<b>ensures</b> result == <a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>();
</code></pre>




<a name="0x1_LibraTimestamp_spec_now_microseconds"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>(): u64 {
<b>global</b>&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).microseconds
}
</code></pre>



<a name="0x1_LibraTimestamp_Specification_now_seconds"></a>

### Function `now_seconds`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_seconds">now_seconds</a>(): u64
</code></pre>




<pre><code>pragma opaque;
<b>include</b> <a href="#0x1_LibraTimestamp_AbortsIfNoTime">AbortsIfNoTime</a>;
<b>ensures</b> result == <a href="#0x1_LibraTimestamp_spec_now_microseconds">spec_now_microseconds</a>() /  MICRO_CONVERSION_FACTOR;
</code></pre>




<a name="0x1_LibraTimestamp_spec_now_seconds"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_spec_now_seconds">spec_now_seconds</a>(): u64 {
<b>global</b>&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).microseconds / MICRO_CONVERSION_FACTOR
}
</code></pre>


Schema specifying that a function aborts if the timer is not published.


<a name="0x1_LibraTimestamp_AbortsIfNoTime"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_AbortsIfNoTime">AbortsIfNoTime</a> {
    <b>aborts_if</b> !<a href="#0x1_LibraTimestamp_spec_timer_initialized">spec_timer_initialized</a>() with Errors::NOT_PUBLISHED;
}
</code></pre>



<a name="0x1_LibraTimestamp_Specification_assert_genesis"></a>

### Function `assert_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_assert_genesis">assert_genesis</a>()
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>include</b> <a href="#0x1_LibraTimestamp_AbortsIfNotGenesis">AbortsIfNotGenesis</a>;
</code></pre>


Helper schema to specify that a function aborts if not in genesis.


<a name="0x1_LibraTimestamp_AbortsIfNotGenesis"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_AbortsIfNotGenesis">AbortsIfNotGenesis</a> {
    <b>aborts_if</b> !<a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>() with Errors::INVALID_STATE;
}
</code></pre>



<a name="0x1_LibraTimestamp_Specification_assert_operating"></a>

### Function `assert_operating`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_assert_operating">assert_operating</a>()
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>include</b> <a href="#0x1_LibraTimestamp_AbortsIfNotOperating">AbortsIfNotOperating</a>;
</code></pre>


Helper schema to specify that a function aborts if not operating.


<a name="0x1_LibraTimestamp_AbortsIfNotOperating"></a>


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_AbortsIfNotOperating">AbortsIfNotOperating</a> {
    <b>aborts_if</b> !<a href="#0x1_LibraTimestamp_is_operating">is_operating</a>() with Errors::INVALID_STATE;
}
</code></pre>

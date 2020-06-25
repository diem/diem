
<a name="0x1_LibraTimestamp"></a>

# Module `0x1::LibraTimestamp`

### Table of Contents

-  [Resource `CurrentTimeMicroseconds`](#0x1_LibraTimestamp_CurrentTimeMicroseconds)
-  [Resource `TimeHasStarted`](#0x1_LibraTimestamp_TimeHasStarted)
-  [Function `initialize`](#0x1_LibraTimestamp_initialize)
-  [Function `set_time_has_started`](#0x1_LibraTimestamp_set_time_has_started)
-  [Function `update_global_time`](#0x1_LibraTimestamp_update_global_time)
-  [Function `now_microseconds`](#0x1_LibraTimestamp_now_microseconds)
-  [Function `is_genesis`](#0x1_LibraTimestamp_is_genesis)
-  [Function `is_not_initialized`](#0x1_LibraTimestamp_is_not_initialized)
-  [Function `assert_is_genesis`](#0x1_LibraTimestamp_assert_is_genesis)
-  [Specification](#0x1_LibraTimestamp_Specification)
    -  [Module specification](#0x1_LibraTimestamp_@Module_specification)
        -  [Persistence of Initialization](#0x1_LibraTimestamp_@Persistence_of_Initialization)
        -  [Global Clock Time Progression](#0x1_LibraTimestamp_@Global_Clock_Time_Progression)
    -  [Function `initialize`](#0x1_LibraTimestamp_Specification_initialize)
    -  [Function `set_time_has_started`](#0x1_LibraTimestamp_Specification_set_time_has_started)
    -  [Function `update_global_time`](#0x1_LibraTimestamp_Specification_update_global_time)
    -  [Function `now_microseconds`](#0x1_LibraTimestamp_Specification_now_microseconds)
    -  [Function `is_genesis`](#0x1_LibraTimestamp_Specification_is_genesis)
    -  [Function `is_not_initialized`](#0x1_LibraTimestamp_Specification_is_not_initialized)

This module keeps a global wall clock that stores the current Unix time in microseconds.
It interacts with the other modules in the following ways:

* Genesis: to initialize the timestamp
* VASP: to keep track of when credentials expire
* LibraSystem, LibraAccount, LibraConfig: to check if the current state is in the genesis state
* LibraBlock: to reach consensus on the global wall clock time
* AccountLimits: to limit the time of account limits
* LibraTransactionTimeout: to determine whether a transaction is still valid


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

<a name="0x1_LibraTimestamp_initialize"></a>

## Function `initialize`

Initializes the global wall clock time resource. This can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(association: &signer) {
    // Operational constraint, only callable by the Association address
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 1);

    // TODO: Should the initialized value be passed in <b>to</b> genesis?
    <b>let</b> timer = <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> { microseconds: 0 };
    move_to(association, timer);
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_set_time_has_started"></a>

## Function `set_time_has_started`

Marks that time has started and genesis has finished. This can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_set_time_has_started">set_time_has_started</a>(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_set_time_has_started">set_time_has_started</a>(association: &signer) <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 1);

    // Current time must have been initialized.
    <b>assert</b>(
        exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) && <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>() == 0,
        2
    );
    move_to(association, <a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>{});
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
    // Can only be invoked by LibraVM privilege.
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), 33);

    <b>let</b> global_timer = borrow_global_mut&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <b>if</b> (proposer == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>()) {
        // NIL block with null address <b>as</b> proposer. Timestamp must be equal.
        <b>assert</b>(timestamp == global_timer.microseconds, 5001);
    } <b>else</b> {
        // Normal block. Time must advance
        <b>assert</b>(global_timer.microseconds &lt; timestamp, 5001);
    };
    global_timer.microseconds = timestamp;
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_now_microseconds"></a>

## Function `now_microseconds`

Gets the timestamp representing
<code>now</code> in microseconds.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64 <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    borrow_global&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).microseconds
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_is_genesis"></a>

## Function `is_genesis`

Helper function to determine if the blockchain is in genesis state.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(): bool {
    !exists&lt;<a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_is_not_initialized"></a>

## Function `is_not_initialized`

Helper function to determine whether the CurrentTime has been initialized.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_not_initialized">is_not_initialized</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_not_initialized">is_not_initialized</a>(): bool <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
   !exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) || <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>() == 0
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_assert_is_genesis"></a>

## Function `assert_is_genesis`

Helper function which aborts if not in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_assert_is_genesis">assert_is_genesis</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_assert_is_genesis">assert_is_genesis</a>() {
    <b>assert</b>(<a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(), 0);
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_Specification"></a>

## Specification


<a name="0x1_LibraTimestamp_@Module_specification"></a>

### Module specification


Verify all functions in this module.


<pre><code>pragma verify = <b>true</b>;
</code></pre>


Specification version of the
<code><a href="#0x1_LibraTimestamp_is_genesis">Self::is_genesis</a></code> function.


<a name="0x1_LibraTimestamp_spec_is_genesis"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_spec_is_genesis">spec_is_genesis</a>(): bool {
    !exists&lt;<a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>


True if the association root account has a CurrentTimeMicroseconds.


<a name="0x1_LibraTimestamp_root_ctm_initialized"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>(): bool {
    exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>


Auxiliary function to get the association's Unix time in microseconds.


<a name="0x1_LibraTimestamp_assoc_unix_time"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>(): u64 {
    <b>global</b>&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>()).microseconds
}
</code></pre>



<a name="0x1_LibraTimestamp_@Persistence_of_Initialization"></a>

#### Persistence of Initialization



<a name="0x1_LibraTimestamp_InitializationPersists"></a>

If the
<code><a href="#0x1_LibraTimestamp_TimeHasStarted">TimeHasStarted</a></code> resource is initialized and we finished genesis, we can never enter genesis again.
Note that this is an important safety property since during genesis, we are allowed to perform certain
operations which should never be allowed in normal on-chain execution.


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_InitializationPersists">InitializationPersists</a> {
    <b>ensures</b> <b>old</b>(!<a href="#0x1_LibraTimestamp_spec_is_genesis">spec_is_genesis</a>()) ==&gt; !<a href="#0x1_LibraTimestamp_spec_is_genesis">spec_is_genesis</a>();
}
</code></pre>


If the
<code><a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a></code> resource is initialized, it stays initialized.


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_InitializationPersists">InitializationPersists</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>()) ==&gt; <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_LibraTimestamp_InitializationPersists">InitializationPersists</a> <b>to</b> *;
</code></pre>



<a name="0x1_LibraTimestamp_@Global_Clock_Time_Progression"></a>

#### Global Clock Time Progression



<a name="0x1_LibraTimestamp_GlobalWallClockIsMonotonic"></a>

The global wall clock time never decreases.


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_GlobalWallClockIsMonotonic">GlobalWallClockIsMonotonic</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>()) ==&gt; (<b>old</b>(<a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>()) &lt;= <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>());
}
</code></pre>




<pre><code><b>apply</b> <a href="#0x1_LibraTimestamp_GlobalWallClockIsMonotonic">GlobalWallClockIsMonotonic</a> <b>to</b> *;
</code></pre>



<a name="0x1_LibraTimestamp_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(association: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(association) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
<b>aborts_if</b> <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
<b>ensures</b> <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
<b>ensures</b> <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>() == 0;
</code></pre>



<a name="0x1_LibraTimestamp_Specification_set_time_has_started"></a>

### Function `set_time_has_started`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_set_time_has_started">set_time_has_started</a>(association: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(association) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
<b>aborts_if</b> !<a href="#0x1_LibraTimestamp_spec_is_genesis">spec_is_genesis</a>();
<b>aborts_if</b> !<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
<b>aborts_if</b> <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>() != 0;
<b>ensures</b> !<a href="#0x1_LibraTimestamp_spec_is_genesis">spec_is_genesis</a>();
</code></pre>



<a name="0x1_LibraTimestamp_Specification_update_global_time"></a>

### Function `update_global_time`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_update_global_time">update_global_time</a>(account: &signer, proposer: address, timestamp: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">CoreAddresses::SPEC_VM_RESERVED_ADDRESS</a>();
<b>aborts_if</b> !<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
<b>aborts_if</b> (proposer == <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">CoreAddresses::SPEC_VM_RESERVED_ADDRESS</a>()) && (timestamp != <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>());
<b>aborts_if</b> (proposer != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">CoreAddresses::SPEC_VM_RESERVED_ADDRESS</a>()) && !(timestamp &gt; <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>());
<b>ensures</b> <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>() == timestamp;
</code></pre>



<a name="0x1_LibraTimestamp_Specification_now_microseconds"></a>

### Function `now_microseconds`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>());
<b>ensures</b> result == <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>();
</code></pre>



<a name="0x1_LibraTimestamp_Specification_is_genesis"></a>

### Function `is_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="#0x1_LibraTimestamp_spec_is_genesis">spec_is_genesis</a>();
</code></pre>



<a name="0x1_LibraTimestamp_Specification_is_not_initialized"></a>

### Function `is_not_initialized`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_not_initialized">is_not_initialized</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == !<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>() || <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>() == 0;
</code></pre>

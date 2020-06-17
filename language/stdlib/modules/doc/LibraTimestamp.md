
<a name="0x1_LibraTimestamp"></a>

# Module `0x1::LibraTimestamp`

### Table of Contents

-  [Struct `CurrentTimeMicroseconds`](#0x1_LibraTimestamp_CurrentTimeMicroseconds)
-  [Function `initialize`](#0x1_LibraTimestamp_initialize)
-  [Function `update_global_time`](#0x1_LibraTimestamp_update_global_time)
-  [Function `now_microseconds`](#0x1_LibraTimestamp_now_microseconds)
-  [Function `is_genesis`](#0x1_LibraTimestamp_is_genesis)
-  [Specification](#0x1_LibraTimestamp_Specification)
    -  [Module specification](#0x1_LibraTimestamp_@Module_specification)
        -  [Management of the global wall clock time](#0x1_LibraTimestamp_@Management_of_the_global_wall_clock_time)
        -  [Global wall clock time progression](#0x1_LibraTimestamp_@Global_wall_clock_time_progression)
    -  [Function `initialize`](#0x1_LibraTimestamp_Specification_initialize)
    -  [Function `update_global_time`](#0x1_LibraTimestamp_Specification_update_global_time)
    -  [Function `now_microseconds`](#0x1_LibraTimestamp_Specification_now_microseconds)
    -  [Function `is_genesis`](#0x1_LibraTimestamp_Specification_is_genesis)



<a name="0x1_LibraTimestamp_CurrentTimeMicroseconds"></a>

## Struct `CurrentTimeMicroseconds`



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

<a name="0x1_LibraTimestamp_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(association: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(association: &signer) {
    // Only callable by the <a href="Association.md#0x1_Association">Association</a> address
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(association) == <a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>(), 1);

    // TODO: Should the initialized value be passed in <b>to</b> genesis?
    <b>let</b> timer = <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> { microseconds: 0 };
    move_to(association, timer);
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_update_global_time"></a>

## Function `update_global_time`



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

    <b>let</b> global_timer = borrow_global_mut&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>());
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



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64 <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    borrow_global&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>()).microseconds
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_is_genesis"></a>

## Function `is_genesis`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(): bool <b>acquires</b> <a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a> {
    !exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_ASSOCIATION_ROOT_ADDRESS">CoreAddresses::ASSOCIATION_ROOT_ADDRESS</a>()) || <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>() == 0
}
</code></pre>



</details>

<a name="0x1_LibraTimestamp_Specification"></a>

## Specification


**************** SPECIFICATIONS ****************

This module keeps a global wall clock that stores the current Unix time in microseconds.
It interacts with the other modules in the following ways:
* Genesis: to initialize the timestamp
* VASP: to keep track of when credentials expire
* LibraSystem, LibraAccount, LibraConfig: to check if the current state is the genesis state
* LibraBlock: to reach consensus on the global wall clock time
* AccountLimits: to limit the time of account limits
* LibraTransactionTimeout: to determine whether a transaction is still valid


<a name="0x1_LibraTimestamp_@Module_specification"></a>

### Module specification


Verify all functions in this module.


<pre><code>pragma verify = <b>true</b>;
</code></pre>


The association root address.


<a name="0x1_LibraTimestamp_root_address"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_root_address">root_address</a>(): address {
    0xA550C18
}
</code></pre>


The nil account with VM privilege.


<a name="0x1_LibraTimestamp_null_address"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_null_address">null_address</a>(): address {
    0x0
}
</code></pre>


True if the association root account has a CurrentTimeMicroseconds.


<a name="0x1_LibraTimestamp_root_ctm_initialized"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>(): bool {
    exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="#0x1_LibraTimestamp_root_address">root_address</a>())
}
</code></pre>


Auxiliary function to get the association's Unix time in microseconds.


<a name="0x1_LibraTimestamp_assoc_unix_time"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>(): u64 {
    <b>global</b>&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="#0x1_LibraTimestamp_root_address">root_address</a>()).microseconds
}
</code></pre>



<a name="0x1_LibraTimestamp_@Management_of_the_global_wall_clock_time"></a>

#### Management of the global wall clock time


**Informally:** Only the root address has a CurrentTimeMicroseconds struct.


<a name="0x1_LibraTimestamp_only_root_addr_has_ctm"></a>


<pre><code><b>define</b> <a href="#0x1_LibraTimestamp_only_root_addr_has_ctm">only_root_addr_has_ctm</a>(): bool {
    (forall addr: address:
        exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(addr)
            ==&gt; addr == <a href="#0x1_LibraTimestamp_root_address">root_address</a>())
}
</code></pre>




<a name="0x1_LibraTimestamp_OnlyRootAddressHasTimestamp"></a>

Base case of the induction step before the root
account is initialized with the CurrentTimeMicroseconds.

**Informally:** If the root account hasn't been initialized with
CurrentTimeMicroseconds, then it should not exist.


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_OnlyRootAddressHasTimestamp">OnlyRootAddressHasTimestamp</a> {
    <b>invariant</b> <b>module</b> !<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>()
                        ==&gt; (forall addr: address: !exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(addr));
}
</code></pre>


Induction hypothesis for invariant after initialization.

**Informally:** Only the association account has a timestamp
<code><a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a></code>.


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_OnlyRootAddressHasTimestamp">OnlyRootAddressHasTimestamp</a> {
    <b>invariant</b> <b>module</b> <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>() ==&gt; <a href="#0x1_LibraTimestamp_only_root_addr_has_ctm">only_root_addr_has_ctm</a>();
}
</code></pre>


**Informally:** If the CurrentTimeMicroseconds is initialized, it stays initialized.


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_OnlyRootAddressHasTimestamp">OnlyRootAddressHasTimestamp</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>()) ==&gt; <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
}
</code></pre>



Apply
<code><a href="#0x1_LibraTimestamp_OnlyRootAddressHasTimestamp">OnlyRootAddressHasTimestamp</a></code> to all functions.
No other functions should contain the CurrentTimeMicroseconds struct.


<pre><code><b>apply</b> <a href="#0x1_LibraTimestamp_OnlyRootAddressHasTimestamp">OnlyRootAddressHasTimestamp</a> <b>to</b> *;
</code></pre>




<a name="0x1_LibraTimestamp_@Global_wall_clock_time_progression"></a>

#### Global wall clock time progression



<a name="0x1_LibraTimestamp_GlobalWallClockIsMonotonic"></a>

**Informally:** The global wall clock time never decreases.


<pre><code><b>schema</b> <a href="#0x1_LibraTimestamp_GlobalWallClockIsMonotonic">GlobalWallClockIsMonotonic</a> {
    <b>ensures</b> <b>old</b>(<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>()) ==&gt; (<b>old</b>(<a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>()) &lt;= <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>());
}
</code></pre>



Apply
<code><a href="#0x1_LibraTimestamp_GlobalWallClockIsMonotonic">GlobalWallClockIsMonotonic</a></code> to all functions.
The global wall clock time should only increase.


<pre><code><b>apply</b> <a href="#0x1_LibraTimestamp_GlobalWallClockIsMonotonic">GlobalWallClockIsMonotonic</a> <b>to</b> *;
</code></pre>




<a name="0x1_LibraTimestamp_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_initialize">initialize</a>(association: &signer)
</code></pre>



The association / root account creates a timestamp struct
under its account during initialization.


<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(association) != <a href="#0x1_LibraTimestamp_root_address">root_address</a>();
<b>aborts_if</b> <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
<b>ensures</b> <a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
<b>ensures</b> <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>() == 0;
</code></pre>



<a name="0x1_LibraTimestamp_Specification_update_global_time"></a>

### Function `update_global_time`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_update_global_time">update_global_time</a>(account: &signer, proposer: address, timestamp: u64)
</code></pre>



Used to update every proposers' global wall
clock time by consensus.


<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_get_address">Signer::get_address</a>(account) != <a href="#0x1_LibraTimestamp_null_address">null_address</a>();
<b>aborts_if</b> !<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>();
<b>aborts_if</b> (proposer == <a href="#0x1_LibraTimestamp_null_address">null_address</a>()) && (timestamp != <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>());
<b>aborts_if</b> (proposer != <a href="#0x1_LibraTimestamp_null_address">null_address</a>()) && !(timestamp &gt; <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>());
<b>ensures</b> <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>() == timestamp;
</code></pre>



<a name="0x1_LibraTimestamp_Specification_now_microseconds"></a>

### Function `now_microseconds`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_now_microseconds">now_microseconds</a>(): u64
</code></pre>



Returns the global wall clock time if it exists.


<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_LibraTimestamp_CurrentTimeMicroseconds">CurrentTimeMicroseconds</a>&gt;(<a href="#0x1_LibraTimestamp_root_address">root_address</a>());
<b>ensures</b> result == <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>();
</code></pre>



<a name="0x1_LibraTimestamp_Specification_is_genesis"></a>

### Function `is_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraTimestamp_is_genesis">is_genesis</a>(): bool
</code></pre>



Returns whether or not it is the beginning of time.


<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> (result == !<a href="#0x1_LibraTimestamp_root_ctm_initialized">root_ctm_initialized</a>() || <a href="#0x1_LibraTimestamp_assoc_unix_time">assoc_unix_time</a>() == 0);
</code></pre>

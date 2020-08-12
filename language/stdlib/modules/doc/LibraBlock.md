
<a name="0x1_LibraBlock"></a>

# Module `0x1::LibraBlock`

### Table of Contents

-  [Resource `BlockMetadata`](#0x1_LibraBlock_BlockMetadata)
-  [Struct `NewBlockEvent`](#0x1_LibraBlock_NewBlockEvent)
-  [Const `EBLOCK_METADATA`](#0x1_LibraBlock_EBLOCK_METADATA)
-  [Const `ESENDER_NOT_VM`](#0x1_LibraBlock_ESENDER_NOT_VM)
-  [Const `EVM_OR_VALIDATOR`](#0x1_LibraBlock_EVM_OR_VALIDATOR)
-  [Function `initialize_block_metadata`](#0x1_LibraBlock_initialize_block_metadata)
-  [Function `is_initialized`](#0x1_LibraBlock_is_initialized)
-  [Function `block_prologue`](#0x1_LibraBlock_block_prologue)
-  [Function `get_current_block_height`](#0x1_LibraBlock_get_current_block_height)
-  [Specification](#0x1_LibraBlock_Specification)
    -  [Function `initialize_block_metadata`](#0x1_LibraBlock_Specification_initialize_block_metadata)
    -  [Function `block_prologue`](#0x1_LibraBlock_Specification_block_prologue)



<a name="0x1_LibraBlock_BlockMetadata"></a>

## Resource `BlockMetadata`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>height: u64</code>
</dt>
<dd>
 Height of the current block
 TODO: should we keep the height?
</dd>
<dt>

<code>new_block_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraBlock_NewBlockEvent">LibraBlock::NewBlockEvent</a>&gt;</code>
</dt>
<dd>
 Handle where events with the time of new blocks are emitted
</dd>
</dl>


</details>

<a name="0x1_LibraBlock_NewBlockEvent"></a>

## Struct `NewBlockEvent`



<pre><code><b>struct</b> <a href="#0x1_LibraBlock_NewBlockEvent">NewBlockEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>round: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>proposer: address</code>
</dt>
<dd>

</dd>
<dt>

<code>previous_block_votes: vector&lt;address&gt;</code>
</dt>
<dd>

</dd>
<dt>

<code>time_microseconds: u64</code>
</dt>
<dd>
 On-chain time during  he block at the given height
</dd>
</dl>


</details>

<a name="0x1_LibraBlock_EBLOCK_METADATA"></a>

## Const `EBLOCK_METADATA`



<pre><code><b>const</b> EBLOCK_METADATA: u64 = 0;
</code></pre>



<a name="0x1_LibraBlock_ESENDER_NOT_VM"></a>

## Const `ESENDER_NOT_VM`



<pre><code><b>const</b> ESENDER_NOT_VM: u64 = 2;
</code></pre>



<a name="0x1_LibraBlock_EVM_OR_VALIDATOR"></a>

## Const `EVM_OR_VALIDATOR`



<pre><code><b>const</b> EVM_OR_VALIDATOR: u64 = 3;
</code></pre>



<a name="0x1_LibraBlock_initialize_block_metadata"></a>

## Function `initialize_block_metadata`

This can only be invoked by the Association address, and only a single time.
Currently, it is invoked in the genesis transaction


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    // Operational constraint, only callable by the Association address
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_libra_root">CoreAddresses::assert_libra_root</a>(account);

    <b>assert</b>(!<a href="#0x1_LibraBlock_is_initialized">is_initialized</a>(), <a href="Errors.md#0x1_Errors_already_published">Errors::already_published</a>(EBLOCK_METADATA));
    move_to&lt;<a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(
        account,
        <a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a> {
            height: 0,
            new_block_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_LibraBlock_NewBlockEvent">Self::NewBlockEvent</a>&gt;(account),
        }
    );
}
</code></pre>



</details>

<a name="0x1_LibraBlock_is_initialized"></a>

## Function `is_initialized`

Helper function to determine whether this module has been initialized.


<pre><code><b>fun</b> <a href="#0x1_LibraBlock_is_initialized">is_initialized</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraBlock_is_initialized">is_initialized</a>(): bool {
    exists&lt;<a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_LibraBlock_block_prologue"></a>

## Function `block_prologue`

Set the metadata for the current block.
The runtime always runs this before executing the transactions in a block.
TODO: 1. Make this private, support other metadata
2. Should the previous block votes be provided from BlockMetadata or should it come from the ValidatorSet
Resource?


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_block_prologue">block_prologue</a>(vm: &signer, round: u64, timestamp: u64, previous_block_votes: vector&lt;address&gt;, proposer: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_block_prologue">block_prologue</a>(
    vm: &signer,
    round: u64,
    timestamp: u64,
    previous_block_votes: vector&lt;address&gt;,
    proposer: address
) <b>acquires</b> <a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a> {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();
    // Operational constraint: can only be invoked by the VM.
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_vm">CoreAddresses::assert_vm</a>(vm);

    // Authorization
    <b>assert</b>(
        proposer == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>() || <a href="LibraSystem.md#0x1_LibraSystem_is_validator">LibraSystem::is_validator</a>(proposer),
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(EVM_OR_VALIDATOR)
    );

    <b>let</b> block_metadata_ref = borrow_global_mut&lt;<a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_update_global_time">LibraTimestamp::update_global_time</a>(vm, proposer, timestamp);
    block_metadata_ref.height = block_metadata_ref.height + 1;
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x1_LibraBlock_NewBlockEvent">NewBlockEvent</a>&gt;(
        &<b>mut</b> block_metadata_ref.new_block_events,
        <a href="#0x1_LibraBlock_NewBlockEvent">NewBlockEvent</a> {
            round: round,
            proposer: proposer,
            previous_block_votes: previous_block_votes,
            time_microseconds: timestamp,
        }
    );
    // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
}
</code></pre>



</details>

<a name="0x1_LibraBlock_get_current_block_height"></a>

## Function `get_current_block_height`

Get the current block height


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>(): u64 <b>acquires</b> <a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a> {
    borrow_global&lt;<a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).height
}
</code></pre>



</details>

<a name="0x1_LibraBlock_Specification"></a>

## Specification



<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="#0x1_LibraBlock_is_initialized">is_initialized</a>();
</code></pre>



<a name="0x1_LibraBlock_Specification_initialize_block_metadata"></a>

### Function `initialize_block_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotLibraRoot">CoreAddresses::AbortsIfNotLibraRoot</a>;
<b>aborts_if</b> <a href="#0x1_LibraBlock_is_initialized">is_initialized</a>() with Errors::ALREADY_PUBLISHED;
<b>ensures</b> <a href="#0x1_LibraBlock_is_initialized">is_initialized</a>();
<b>ensures</b> <a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>() == 0;
</code></pre>



<a name="0x1_LibraBlock_Specification_block_prologue"></a>

### Function `block_prologue`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_block_prologue">block_prologue</a>(vm: &signer, round: u64, timestamp: u64, previous_block_votes: vector&lt;address&gt;, proposer: address)
</code></pre>




<pre><code><b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotVM">CoreAddresses::AbortsIfNotVM</a>{account: vm};
<b>aborts_if</b> proposer != <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>() && !<a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">LibraSystem::spec_is_validator</a>(proposer)
    with Errors::REQUIRES_ADDRESS;
<b>ensures</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() == timestamp;
<b>ensures</b> <a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>() == <b>old</b>(<a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>()) + 1;
</code></pre>


The below counter overflow is assumed to be excluded from verification of callers.


<pre><code><b>aborts_if</b> [<b>assume</b>] <a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>() + 1 &gt; MAX_U64 with EXECUTION_FAILURE;
</code></pre>




<pre><code>pragma verify;
</code></pre>

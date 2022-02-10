
<a name="0x1_DiemBlock"></a>

# Module `0x1::DiemBlock`

This module defines a struct storing the metadata of the block and new block events.


-  [Resource `BlockMetadata`](#0x1_DiemBlock_BlockMetadata)
-  [Struct `NewBlockEvent`](#0x1_DiemBlock_NewBlockEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize_block_metadata`](#0x1_DiemBlock_initialize_block_metadata)
-  [Function `is_initialized`](#0x1_DiemBlock_is_initialized)
-  [Function `block_prologue`](#0x1_DiemBlock_block_prologue)
-  [Function `get_current_block_height`](#0x1_DiemBlock_get_current_block_height)


<pre><code><b>use</b> <a href="DiemSystem.md#0x1_DiemSystem">0x1::DiemSystem</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="SystemAddresses.md#0x1_SystemAddresses">0x1::SystemAddresses</a>;
</code></pre>



<a name="0x1_DiemBlock_BlockMetadata"></a>

## Resource `BlockMetadata`



<pre><code><b>struct</b> <a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>height: u64</code>
</dt>
<dd>
 Height of the current block
</dd>
<dt>
<code>new_block_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="DiemBlock.md#0x1_DiemBlock_NewBlockEvent">DiemBlock::NewBlockEvent</a>&gt;</code>
</dt>
<dd>
 Handle where events with the time of new blocks are emitted
</dd>
</dl>


</details>

<a name="0x1_DiemBlock_NewBlockEvent"></a>

## Struct `NewBlockEvent`



<pre><code><b>struct</b> <a href="DiemBlock.md#0x1_DiemBlock_NewBlockEvent">NewBlockEvent</a> <b>has</b> drop, store
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
<code>proposer: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>previous_block_votes: vector&lt;<b>address</b>&gt;</code>
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemBlock_EBLOCK_METADATA"></a>

The <code><a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a></code> resource is in an invalid state


<pre><code><b>const</b> <a href="DiemBlock.md#0x1_DiemBlock_EBLOCK_METADATA">EBLOCK_METADATA</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemBlock_EVM_OR_VALIDATOR"></a>

An invalid signer was provided. Expected the signer to be the VM or a Validator.


<pre><code><b>const</b> <a href="DiemBlock.md#0x1_DiemBlock_EVM_OR_VALIDATOR">EVM_OR_VALIDATOR</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemBlock_initialize_block_metadata"></a>

## Function `initialize_block_metadata`

This can only be invoked by the Association address, and only a single time.
Currently, it is invoked in the genesis transaction


<pre><code><b>public</b> <b>fun</b> <a href="DiemBlock.md#0x1_DiemBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemBlock.md#0x1_DiemBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    // Operational constraint, only callable by the Association <b>address</b>
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_core_resource">SystemAddresses::assert_core_resource</a>(account);

    <b>assert</b>!(!<a href="DiemBlock.md#0x1_DiemBlock_is_initialized">is_initialized</a>(), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemBlock.md#0x1_DiemBlock_EBLOCK_METADATA">EBLOCK_METADATA</a>));
    <b>move_to</b>&lt;<a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a>&gt;(
        account,
        <a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a> {
            height: 0,
            new_block_events: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="DiemBlock.md#0x1_DiemBlock_NewBlockEvent">Self::NewBlockEvent</a>&gt;(account),
        }
    );
}
</code></pre>



</details>

<a name="0x1_DiemBlock_is_initialized"></a>

## Function `is_initialized`

Helper function to determine whether this module has been initialized.


<pre><code><b>fun</b> <a href="DiemBlock.md#0x1_DiemBlock_is_initialized">is_initialized</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemBlock.md#0x1_DiemBlock_is_initialized">is_initialized</a>(): bool {
    <b>exists</b>&lt;<a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a>&gt;(@CoreResources)
}
</code></pre>



</details>

<a name="0x1_DiemBlock_block_prologue"></a>

## Function `block_prologue`

Set the metadata for the current block.
The runtime always runs this before executing the transactions in a block.


<pre><code><b>fun</b> <a href="DiemBlock.md#0x1_DiemBlock_block_prologue">block_prologue</a>(vm: signer, round: u64, timestamp: u64, previous_block_votes: vector&lt;<b>address</b>&gt;, proposer: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="DiemBlock.md#0x1_DiemBlock_block_prologue">block_prologue</a>(
    vm: signer,
    round: u64,
    timestamp: u64,
    previous_block_votes: vector&lt;<b>address</b>&gt;,
    proposer: <b>address</b>
) <b>acquires</b> <a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a> {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_operating">DiemTimestamp::assert_operating</a>();
    // Operational constraint: can only be invoked by the VM.
    <a href="SystemAddresses.md#0x1_SystemAddresses_assert_vm">SystemAddresses::assert_vm</a>(&vm);

    // Authorization
    <b>assert</b>!(
        proposer == @VMReserved || <a href="DiemSystem.md#0x1_DiemSystem_is_validator">DiemSystem::is_validator</a>(proposer),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="DiemBlock.md#0x1_DiemBlock_EVM_OR_VALIDATOR">EVM_OR_VALIDATOR</a>)
    );

    <b>let</b> block_metadata_ref = <b>borrow_global_mut</b>&lt;<a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a>&gt;(@CoreResources);
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_update_global_time">DiemTimestamp::update_global_time</a>(&vm, proposer, timestamp);
    block_metadata_ref.height = block_metadata_ref.height + 1;
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="DiemBlock.md#0x1_DiemBlock_NewBlockEvent">NewBlockEvent</a>&gt;(
        &<b>mut</b> block_metadata_ref.new_block_events,
        <a href="DiemBlock.md#0x1_DiemBlock_NewBlockEvent">NewBlockEvent</a> {
            round,
            proposer,
            previous_block_votes,
            time_microseconds: timestamp,
        }
    );
}
</code></pre>



</details>

<a name="0x1_DiemBlock_get_current_block_height"></a>

## Function `get_current_block_height`

Get the current block height


<pre><code><b>public</b> <b>fun</b> <a href="DiemBlock.md#0x1_DiemBlock_get_current_block_height">get_current_block_height</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemBlock.md#0x1_DiemBlock_get_current_block_height">get_current_block_height</a>(): u64 <b>acquires</b> <a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a> {
    <b>assert</b>!(<a href="DiemBlock.md#0x1_DiemBlock_is_initialized">is_initialized</a>(), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemBlock.md#0x1_DiemBlock_EBLOCK_METADATA">EBLOCK_METADATA</a>));
    <b>borrow_global</b>&lt;<a href="DiemBlock.md#0x1_DiemBlock_BlockMetadata">BlockMetadata</a>&gt;(@CoreResources).height
}
</code></pre>



</details>

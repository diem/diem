
<a name="0x1_LibraBlock"></a>

# Module `0x1::LibraBlock`

### Table of Contents

-  [Resource `BlockMetadata`](#0x1_LibraBlock_BlockMetadata)
-  [Struct `NewBlockEvent`](#0x1_LibraBlock_NewBlockEvent)
-  [Function `initialize_block_metadata`](#0x1_LibraBlock_initialize_block_metadata)
-  [Function `block_prologue`](#0x1_LibraBlock_block_prologue)
-  [Function `process_block_prologue`](#0x1_LibraBlock_process_block_prologue)
-  [Function `get_current_block_height`](#0x1_LibraBlock_get_current_block_height)
-  [Specification](#0x1_LibraBlock_Specification)
    -  [Function `initialize_block_metadata`](#0x1_LibraBlock_Specification_initialize_block_metadata)
    -  [Function `block_prologue`](#0x1_LibraBlock_Specification_block_prologue)
    -  [Function `process_block_prologue`](#0x1_LibraBlock_Specification_process_block_prologue)
    -  [Function `get_current_block_height`](#0x1_LibraBlock_Specification_get_current_block_height)



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

<a name="0x1_LibraBlock_initialize_block_metadata"></a>

## Function `initialize_block_metadata`

This can only be invoked by the Association address, and only a single time.
Currently, it is invoked in the genesis transaction


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer) {
    <b>assert</b>(<a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>(), ENOT_GENESIS);
    // Operational constraint, only callable by the Association address
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), EINVALID_SINGLETON_ADDRESS);

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
    // Can only be invoked by LibraVM privilege.
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(vm) == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), ESENDER_NOT_VM);

    <a href="#0x1_LibraBlock_process_block_prologue">process_block_prologue</a>(vm,  round, timestamp, previous_block_votes, proposer);

    // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
}
</code></pre>



</details>

<a name="0x1_LibraBlock_process_block_prologue"></a>

## Function `process_block_prologue`

Update the BlockMetadata resource with the new blockmetada coming from the consensus.


<pre><code><b>fun</b> <a href="#0x1_LibraBlock_process_block_prologue">process_block_prologue</a>(vm: &signer, round: u64, timestamp: u64, previous_block_votes: vector&lt;address&gt;, proposer: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_LibraBlock_process_block_prologue">process_block_prologue</a>(
    vm: &signer,
    round: u64,
    timestamp: u64,
    previous_block_votes: vector&lt;address&gt;,
    proposer: address
) <b>acquires</b> <a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a> {
    <b>let</b> block_metadata_ref = borrow_global_mut&lt;<a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>());

    <b>if</b>(proposer != <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>()) <b>assert</b>(<a href="LibraSystem.md#0x1_LibraSystem_is_validator">LibraSystem::is_validator</a>(proposer), EPROPOSER_NOT_A_VALIDATOR);
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


<a name="0x1_LibraBlock_Specification_initialize_block_metadata"></a>

### Function `initialize_block_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_is_genesis">LibraTimestamp::spec_is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>();
<b>aborts_if</b> <a href="#0x1_LibraBlock_spec_is_initialized">spec_is_initialized</a>();
<b>ensures</b> <a href="#0x1_LibraBlock_spec_is_initialized">spec_is_initialized</a>();
<b>ensures</b> <a href="#0x1_LibraBlock_spec_block_height">spec_block_height</a>() == 0;
</code></pre>




<a name="0x1_LibraBlock_spec_is_initialized"></a>


<pre><code><b>define</b> <a href="#0x1_LibraBlock_spec_is_initialized">spec_is_initialized</a>(): bool {
    exists&lt;<a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>())
}
</code></pre>



<a name="0x1_LibraBlock_Specification_block_prologue"></a>

### Function `block_prologue`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_block_prologue">block_prologue</a>(vm: &signer, round: u64, timestamp: u64, previous_block_votes: vector&lt;address&gt;, proposer: address)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(vm) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">CoreAddresses::SPEC_VM_RESERVED_ADDRESS</a>();
<b>ensures</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() == timestamp;
<b>ensures</b> <a href="#0x1_LibraBlock_spec_block_height">spec_block_height</a>() == <b>old</b>(<a href="#0x1_LibraBlock_spec_block_height">spec_block_height</a>()) + 10;
</code></pre>




<a name="0x1_LibraBlock_spec_block_height"></a>


<pre><code><b>define</b> <a href="#0x1_LibraBlock_spec_block_height">spec_block_height</a>(): u64 {
    <b>global</b>&lt;<a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_LIBRA_ROOT_ADDRESS">CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS</a>()).height
}
</code></pre>



<a name="0x1_LibraBlock_Specification_process_block_prologue"></a>

### Function `process_block_prologue`


<pre><code><b>fun</b> <a href="#0x1_LibraBlock_process_block_prologue">process_block_prologue</a>(vm: &signer, round: u64, timestamp: u64, previous_block_votes: vector&lt;address&gt;, proposer: address)
</code></pre>




<pre><code>pragma assume_no_abort_from_here = <b>true</b>, opaque = <b>true</b>;
<b>aborts_if</b> !<a href="#0x1_LibraBlock_spec_is_initialized">spec_is_initialized</a>();
<b>aborts_if</b> proposer != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">CoreAddresses::SPEC_VM_RESERVED_ADDRESS</a>()
    && !<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraSystem.md#0x1_LibraSystem_LibraSystem">LibraSystem::LibraSystem</a>&gt;();
<b>aborts_if</b> proposer != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_VM_RESERVED_ADDRESS">CoreAddresses::SPEC_VM_RESERVED_ADDRESS</a>()
    && !<a href="LibraSystem.md#0x1_LibraSystem_spec_is_validator">LibraSystem::spec_is_validator</a>(proposer);
<b>aborts_if</b> <a href="#0x1_LibraBlock_spec_block_height">spec_block_height</a>() + 1  &gt; max_u64();
<b>ensures</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_spec_now_microseconds">LibraTimestamp::spec_now_microseconds</a>() == timestamp;
<b>ensures</b> <a href="#0x1_LibraBlock_spec_block_height">spec_block_height</a>() == <b>old</b>(<a href="#0x1_LibraBlock_spec_block_height">spec_block_height</a>()) + 1;
</code></pre>



<a name="0x1_LibraBlock_Specification_get_current_block_height"></a>

### Function `get_current_block_height`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<a href="#0x1_LibraBlock_spec_is_initialized">spec_is_initialized</a>();
<b>ensures</b> result == <a href="#0x1_LibraBlock_spec_block_height">spec_block_height</a>();
</code></pre>




<pre><code>pragma verify = <b>true</b>;
</code></pre>

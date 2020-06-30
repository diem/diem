
<a name="0x1_LibraBlock"></a>

# Module `0x1::LibraBlock`

### Table of Contents

-  [Resource `BlockMetadata`](#0x1_LibraBlock_BlockMetadata)
-  [Struct `NewBlockEvent`](#0x1_LibraBlock_NewBlockEvent)
-  [Function `initialize_block_metadata`](#0x1_LibraBlock_initialize_block_metadata)
-  [Function `block_prologue`](#0x1_LibraBlock_block_prologue)
-  [Function `process_block_prologue`](#0x1_LibraBlock_process_block_prologue)
-  [Function `get_current_block_height`](#0x1_LibraBlock_get_current_block_height)



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

</dd>
<dt>

<code>new_block_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_LibraBlock_NewBlockEvent">LibraBlock::NewBlockEvent</a>&gt;</code>
</dt>
<dd>

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

</dd>
</dl>


</details>

<a name="0x1_LibraBlock_initialize_block_metadata"></a>

## Function `initialize_block_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>(account: &signer) {
  // Operational constraint, only callable by the Association address
  <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>(), 1);

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
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(vm) == <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>(), 33);

    <a href="#0x1_LibraBlock_process_block_prologue">process_block_prologue</a>(vm,  round, timestamp, previous_block_votes, proposer);

    // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
}
</code></pre>



</details>

<a name="0x1_LibraBlock_process_block_prologue"></a>

## Function `process_block_prologue`



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

    // TODO: Figure out a story for errors in the system transactions.
    <b>if</b>(proposer != <a href="CoreAddresses.md#0x1_CoreAddresses_VM_RESERVED_ADDRESS">CoreAddresses::VM_RESERVED_ADDRESS</a>()) <b>assert</b>(<a href="LibraSystem.md#0x1_LibraSystem_is_validator">LibraSystem::is_validator</a>(proposer), 5002);
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



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_LibraBlock_get_current_block_height">get_current_block_height</a>(): u64 <b>acquires</b> <a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a> {
  borrow_global&lt;<a href="#0x1_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()).height
}
</code></pre>



</details>

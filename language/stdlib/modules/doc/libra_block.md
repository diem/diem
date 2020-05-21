
<a name="0x0_LibraBlock"></a>

# Module `0x0::LibraBlock`

### Table of Contents

-  [Struct `BlockMetadata`](#0x0_LibraBlock_BlockMetadata)
-  [Struct `NewBlockEvent`](#0x0_LibraBlock_NewBlockEvent)
-  [Function `initialize_block_metadata`](#0x0_LibraBlock_initialize_block_metadata)
-  [Function `block_prologue`](#0x0_LibraBlock_block_prologue)
-  [Function `process_block_prologue`](#0x0_LibraBlock_process_block_prologue)
-  [Function `get_current_block_height`](#0x0_LibraBlock_get_current_block_height)



<a name="0x0_LibraBlock_BlockMetadata"></a>

## Struct `BlockMetadata`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x0_LibraBlock_BlockMetadata">BlockMetadata</a>
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

<code>new_block_events: <a href="event.md#0x0_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x0_LibraBlock_NewBlockEvent">LibraBlock::NewBlockEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x0_LibraBlock_NewBlockEvent"></a>

## Struct `NewBlockEvent`



<pre><code><b>struct</b> <a href="#0x0_LibraBlock_NewBlockEvent">NewBlockEvent</a>
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

<a name="0x0_LibraBlock_initialize_block_metadata"></a>

## Function `initialize_block_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraBlock_initialize_block_metadata">initialize_block_metadata</a>() {
  // Only callable by the <a href="association.md#0x0_Association">Association</a> address
  Transaction::assert(Transaction::sender() == 0xA550C18, 1);

  move_to_sender&lt;<a href="#0x0_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(<a href="#0x0_LibraBlock_BlockMetadata">BlockMetadata</a> {
    height: 0,
    new_block_events: <a href="event.md#0x0_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x0_LibraBlock_NewBlockEvent">Self::NewBlockEvent</a>&gt;(),
  });
}
</code></pre>



</details>

<a name="0x0_LibraBlock_block_prologue"></a>

## Function `block_prologue`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraBlock_block_prologue">block_prologue</a>(round: u64, timestamp: u64, previous_block_votes: vector&lt;address&gt;, proposer: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraBlock_block_prologue">block_prologue</a>(
    round: u64,
    timestamp: u64,
    previous_block_votes: vector&lt;address&gt;,
    proposer: address
) <b>acquires</b> <a href="#0x0_LibraBlock_BlockMetadata">BlockMetadata</a> {
    // Can only be invoked by LibraVM privilege.
    Transaction::assert(Transaction::sender() == 0x0, 33);

    <a href="#0x0_LibraBlock_process_block_prologue">process_block_prologue</a>(round, timestamp, previous_block_votes, proposer);

    // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
}
</code></pre>



</details>

<a name="0x0_LibraBlock_process_block_prologue"></a>

## Function `process_block_prologue`



<pre><code><b>fun</b> <a href="#0x0_LibraBlock_process_block_prologue">process_block_prologue</a>(round: u64, timestamp: u64, previous_block_votes: vector&lt;address&gt;, proposer: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x0_LibraBlock_process_block_prologue">process_block_prologue</a>(
    round: u64,
    timestamp: u64,
    previous_block_votes: vector&lt;address&gt;,
    proposer: address
) <b>acquires</b> <a href="#0x0_LibraBlock_BlockMetadata">BlockMetadata</a> {
    <b>let</b> block_metadata_ref = borrow_global_mut&lt;<a href="#0x0_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(0xA550C18);

    // TODO: Figure out a story for errors in the system transactions.
    <b>if</b>(proposer != 0x0) Transaction::assert(<a href="libra_system.md#0x0_LibraSystem_is_validator">LibraSystem::is_validator</a>(proposer), 5002);
    <a href="libra_time.md#0x0_LibraTimestamp_update_global_time">LibraTimestamp::update_global_time</a>(proposer, timestamp);
    block_metadata_ref.height = block_metadata_ref.height + 1;
    <a href="event.md#0x0_Event_emit_event">Event::emit_event</a>&lt;<a href="#0x0_LibraBlock_NewBlockEvent">NewBlockEvent</a>&gt;(
      &<b>mut</b> block_metadata_ref.new_block_events,
      <a href="#0x0_LibraBlock_NewBlockEvent">NewBlockEvent</a> {
        round: round,
        proposer: proposer,
        previous_block_votes: previous_block_votes,
        time_microseconds: timestamp,
      }
    );
}
</code></pre>



</details>

<a name="0x0_LibraBlock_get_current_block_height"></a>

## Function `get_current_block_height`



<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraBlock_get_current_block_height">get_current_block_height</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x0_LibraBlock_get_current_block_height">get_current_block_height</a>(): u64 <b>acquires</b> <a href="#0x0_LibraBlock_BlockMetadata">BlockMetadata</a> {
  borrow_global&lt;<a href="#0x0_LibraBlock_BlockMetadata">BlockMetadata</a>&gt;(0xA550C18).height
}
</code></pre>



</details>

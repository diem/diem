
<a name="0x1_CoreVersion"></a>

# Module `0x1::CoreVersion`

Maintains the version number for the blockchain.


-  [Resource `VersionChainMarker`](#0x1_CoreVersion_VersionChainMarker)
-  [Resource `CoreVersion`](#0x1_CoreVersion_CoreVersion)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_CoreVersion_initialize)
-  [Function `set`](#0x1_CoreVersion_set)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_CoreVersion_VersionChainMarker"></a>

## Resource `VersionChainMarker`

Marker to be stored under 0x1 during genesis


<pre><code><b>struct</b> <a href="CoreVersion.md#0x1_CoreVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt; <b>has</b> key
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

<a name="0x1_CoreVersion_CoreVersion"></a>

## Resource `CoreVersion`



<pre><code><b>struct</b> <a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>major: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_CoreVersion_ECHAIN_MARKER"></a>

Error with chain marker


<pre><code><b>const</b> <a href="CoreVersion.md#0x1_CoreVersion_ECHAIN_MARKER">ECHAIN_MARKER</a>: u64 = 0;
</code></pre>



<a name="0x1_CoreVersion_ECONFIG"></a>

Error with config


<pre><code><b>const</b> <a href="CoreVersion.md#0x1_CoreVersion_ECONFIG">ECONFIG</a>: u64 = 1;
</code></pre>



<a name="0x1_CoreVersion_EINVALID_MAJOR_VERSION_NUMBER"></a>

Tried to set an invalid major version for the VM. Major versions must be strictly increasing


<pre><code><b>const</b> <a href="CoreVersion.md#0x1_CoreVersion_EINVALID_MAJOR_VERSION_NUMBER">EINVALID_MAJOR_VERSION_NUMBER</a>: u64 = 2;
</code></pre>



<a name="0x1_CoreVersion_initialize"></a>

## Function `initialize`

Publishes the Version config.


<pre><code><b>public</b> <b>fun</b> <a href="CoreVersion.md#0x1_CoreVersion_initialize">initialize</a>&lt;T&gt;(account: &signer, initial_version: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreVersion.md#0x1_CoreVersion_initialize">initialize</a>&lt;T&gt;(account: &signer, initial_version: u64) {
    <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == @ConfigStorage, <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="CoreVersion.md#0x1_CoreVersion_ECONFIG">ECONFIG</a>));

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt;&gt;(@ConfigStorage),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="CoreVersion.md#0x1_CoreVersion_ECHAIN_MARKER">ECHAIN_MARKER</a>)
    );

    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="CoreVersion.md#0x1_CoreVersion_ECONFIG">ECONFIG</a>)
    );
    <b>move_to</b>(
        account,
        <a href="CoreVersion.md#0x1_CoreVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt; {},
    );
    <b>move_to</b>(
        account,
        <a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a> { major: initial_version },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage) <b>with</b> Errors::ALREADY_PUBLISHED;
<b>ensures</b> <b>exists</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage);
<b>ensures</b> <b>global</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage).major == initial_version;
</code></pre>



</details>

<a name="0x1_CoreVersion_set"></a>

## Function `set`

Updates the major version to a larger version.


<pre><code><b>public</b> <b>fun</b> <a href="CoreVersion.md#0x1_CoreVersion_set">set</a>&lt;T&gt;(major: u64, _cap: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="CoreVersion.md#0x1_CoreVersion_set">set</a>&lt;T&gt;(major: u64, _cap: &Cap&lt;T&gt;) <b>acquires</b> <a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt;&gt;(@ConfigStorage), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="CoreVersion.md#0x1_CoreVersion_ECHAIN_MARKER">ECHAIN_MARKER</a>));
    <b>assert</b>!(<b>exists</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="CoreVersion.md#0x1_CoreVersion_ECONFIG">ECONFIG</a>));
    <b>let</b> old_major = *&<b>borrow_global</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage).major;

    <b>assert</b>!(
        old_major &lt; major,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="CoreVersion.md#0x1_CoreVersion_EINVALID_MAJOR_VERSION_NUMBER">EINVALID_MAJOR_VERSION_NUMBER</a>)
    );

    <b>let</b> config = <b>borrow_global_mut</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage);
    config.major = major;
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion_VersionChainMarker">VersionChainMarker</a>&lt;T&gt;&gt;(@ConfigStorage) <b>with</b> Errors::NOT_PUBLISHED;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage) <b>with</b> Errors::NOT_PUBLISHED;
<b>aborts_if</b> <b>global</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage).major &gt;= major <b>with</b> Errors::INVALID_ARGUMENT;
<b>ensures</b> <b>global</b>&lt;<a href="CoreVersion.md#0x1_CoreVersion">CoreVersion</a>&gt;(@ConfigStorage).major == major;
</code></pre>



</details>

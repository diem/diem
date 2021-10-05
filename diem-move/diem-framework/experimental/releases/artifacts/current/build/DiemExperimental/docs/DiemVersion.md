
<a name="0x1_DiemVersion"></a>

# Module `0x1::DiemVersion`

Maintains the version number for the blockchain.


-  [Resource `DiemVersion`](#0x1_DiemVersion_DiemVersion)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_DiemVersion_initialize)
-  [Function `set`](#0x1_DiemVersion_set)
-  [Module Specification](#@Module_Specification_1)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_DiemVersion_DiemVersion"></a>

## Resource `DiemVersion`



<pre><code><b>struct</b> <a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt; has <b>copy</b>, drop, store, key
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


<a name="0x1_DiemVersion_ECONFIG"></a>

Error with config


<pre><code><b>const</b> <a href="DiemVersion.md#0x1_DiemVersion_ECONFIG">ECONFIG</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemVersion_EINVALID_MAJOR_VERSION_NUMBER"></a>

Tried to set an invalid major version for the VM. Major versions must be strictly increasing


<pre><code><b>const</b> <a href="DiemVersion.md#0x1_DiemVersion_EINVALID_MAJOR_VERSION_NUMBER">EINVALID_MAJOR_VERSION_NUMBER</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemVersion_initialize"></a>

## Function `initialize`

Publishes the Version config.


<pre><code><b>public</b> <b>fun</b> <a href="DiemVersion.md#0x1_DiemVersion_initialize">initialize</a>&lt;T&gt;(account: &signer, initial_version: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemVersion.md#0x1_DiemVersion_initialize">initialize</a>&lt;T&gt;(account: &signer, initial_version: u64) {
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_already_published">Errors::already_published</a>(<a href="DiemVersion.md#0x1_DiemVersion_ECONFIG">ECONFIG</a>)
    );
    move_to(
        account,
        <a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt; { major: initial_version },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>let</b> config_addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
<b>aborts_if</b> <b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr) <b>with</b> Errors::ALREADY_PUBLISHED;
<b>ensures</b> <b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr);
<b>ensures</b> <b>global</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr).major == initial_version;
</code></pre>



</details>

<a name="0x1_DiemVersion_set"></a>

## Function `set`

Updates the major version to a larger version.


<pre><code><b>public</b> <b>fun</b> <a href="DiemVersion.md#0x1_DiemVersion_set">set</a>&lt;T&gt;(config_addr: address, _cap: &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_Cap">Capability::Cap</a>&lt;T&gt;, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemVersion.md#0x1_DiemVersion_set">set</a>&lt;T&gt;(config_addr: address, _cap: &Cap&lt;T&gt;, major: u64) <b>acquires</b> <a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr), <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="DiemVersion.md#0x1_DiemVersion_ECONFIG">ECONFIG</a>));
    <b>let</b> old_major = *&borrow_global&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr).major;

    <b>assert</b>!(
        old_major &lt; major,
        <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemVersion.md#0x1_DiemVersion_EINVALID_MAJOR_VERSION_NUMBER">EINVALID_MAJOR_VERSION_NUMBER</a>)
    );

    <b>let</b> config = borrow_global_mut&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr);
    config.major = major;
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr) <b>with</b> Errors::NOT_PUBLISHED;
<b>aborts_if</b> <b>global</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr).major &gt;= major <b>with</b> Errors::INVALID_ARGUMENT;
<b>ensures</b> <b>global</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr).major == major;
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



Version number never decreases


<pre><code><b>invariant</b>&lt;T&gt; <b>update</b> [suspendable]
    <b>forall</b> config_addr: address: <b>old</b>(<b>exists</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr))
        ==&gt; (<b>old</b>(<b>global</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr).major) &lt;= <b>global</b>&lt;<a href="DiemVersion.md#0x1_DiemVersion">DiemVersion</a>&lt;T&gt;&gt;(config_addr).major);
</code></pre>

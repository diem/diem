
<a name="0x1_ExperimentalVersion"></a>

# Module `0x1::ExperimentalVersion`

Maintains the version number for the blockchain.


-  [Function `initialize`](#0x1_ExperimentalVersion_initialize)
-  [Function `set`](#0x1_ExperimentalVersion_set)


<pre><code><b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability">0x1::Capability</a>;
<b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemVersion.md#0x1_DiemVersion">0x1::DiemVersion</a>;
<b>use</b> <a href="ExperimentalChainMarker.md#0x1_ExperimentalChainMarker">0x1::ExperimentalChainMarker</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_ExperimentalVersion_initialize"></a>

## Function `initialize`

Publishes the Version config.


<pre><code><b>public</b> <b>fun</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_initialize">initialize</a>(account: &signer, initial_version: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_initialize">initialize</a>(account: &signer, initial_version: u64) {
//        <b>assert</b>!(<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == @0x42, 0);
    <a href="DiemVersion.md#0x1_DiemVersion_initialize">DiemVersion::initialize</a>&lt;<a href="ExperimentalChainMarker.md#0x1_ExperimentalChainMarker">ExperimentalChainMarker</a>&gt;(account, initial_version);
    <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_create">Capability::create</a>&lt;<a href="ExperimentalChainMarker.md#0x1_ExperimentalChainMarker">ExperimentalChainMarker</a>&gt;(account, &get_chain_marker());
}
</code></pre>



</details>

<a name="0x1_ExperimentalVersion_set"></a>

## Function `set`

Updates the major version to a larger version.


<pre><code><b>public</b> <b>fun</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_set">set</a>(account: &signer, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ExperimentalVersion.md#0x1_ExperimentalVersion_set">set</a>(account: &signer, major: u64) {
    <b>let</b> config_addr = <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <a href="DiemVersion.md#0x1_DiemVersion_set">DiemVersion::set</a>&lt;<a href="ExperimentalChainMarker.md#0x1_ExperimentalChainMarker">ExperimentalChainMarker</a>&gt;(
        config_addr,
        &<a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Capability.md#0x1_Capability_acquire">Capability::acquire</a>(account, &get_chain_marker()),
        major
    );
    <a href="DiemConfig.md#0x1_DiemConfig_reconfigure">DiemConfig::reconfigure</a>(account);
}
</code></pre>



</details>

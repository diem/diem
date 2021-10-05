
<a name="0x1_DiemConsensusConfig"></a>

# Module `0x1::DiemConsensusConfig`

Maintains the consensus config for the Diem blockchain. The config is stored in a
DiemConfig, and may be updated by Diem root.


-  [Struct `DiemConsensusConfig`](#0x1_DiemConsensusConfig_DiemConsensusConfig)
-  [Function `initialize`](#0x1_DiemConsensusConfig_initialize)
-  [Function `set`](#0x1_DiemConsensusConfig_set)
-  [Module Specification](#@Module_Specification_0)
    -  [Access Control](#@Access_Control_1)


<pre><code><b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_DiemConsensusConfig_DiemConsensusConfig"></a>

## Struct `DiemConsensusConfig`



<pre><code><b>struct</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DiemConsensusConfig_initialize"></a>

## Function `initialize`

Publishes the DiemConsensusConfig config.


<pre><code><b>public</b> <b>fun</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_initialize">initialize</a>(dr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_initialize">initialize</a>(dr_account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);
    <a href="DiemConfig.md#0x1_DiemConfig_publish_new_config">DiemConfig::publish_new_config</a>(dr_account, <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a> { config: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>() });
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H12]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_PublishNewConfigAbortsIf">DiemConfig::PublishNewConfigAbortsIf</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_PublishNewConfigEnsures">DiemConfig::PublishNewConfigEnsures</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;{
    payload: <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a> { config: <a href="../../../../../../../experimental/releases/artifacts/current/build/MoveStdlib/docs/Vector.md#0x1_Vector_empty">Vector::empty</a>() }
};
</code></pre>



</details>

<a name="0x1_DiemConsensusConfig_set"></a>

## Function `set`

Allows Diem root to update the config.


<pre><code><b>public</b> <b>fun</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_set">set</a>(dr_account: &signer, config: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_set">set</a>(dr_account: &signer, config: vector&lt;u8&gt;) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);

    <a href="DiemConfig.md#0x1_DiemConfig_set">DiemConfig::set</a>(
        dr_account,
        <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a> { config }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H12]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetAbortsIf">DiemConfig::SetAbortsIf</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;{account: dr_account};
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetEnsures">DiemConfig::SetEnsures</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;{payload: <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a> { config }};
</code></pre>



</details>

<a name="@Module_Specification_0"></a>

## Module Specification



<a name="@Access_Control_1"></a>

### Access Control

The permission "UpdateDiemConsensusConfig" is granted to DiemRoot [[H12]][PERMISSION].


<pre><code><b>invariant</b> [suspendable] <b>forall</b> addr: address
    <b>where</b> <b>exists</b>&lt;<a href="DiemConfig.md#0x1_DiemConfig">DiemConfig</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;&gt;(addr): addr == @DiemRoot;
<b>invariant</b> <b>update</b> [suspendable] <b>old</b>(<a href="DiemConfig.md#0x1_DiemConfig_spec_is_published">DiemConfig::spec_is_published</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;())
    && <a href="DiemConfig.md#0x1_DiemConfig_spec_is_published">DiemConfig::spec_is_published</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;()
    && <b>old</b>(<a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;()) != <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;()
        ==&gt; <a href="Roles.md#0x1_Roles_spec_signed_by_diem_root_role">Roles::spec_signed_by_diem_root_role</a>();
</code></pre>


Only "set" can modify the DiemConsensusConfig config [[H12]][PERMISSION]


<a name="0x1_DiemConsensusConfig_DiemConsensusConfigRemainsSame"></a>


<pre><code><b>schema</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_DiemConsensusConfigRemainsSame">DiemConsensusConfigRemainsSame</a> {
    <b>ensures</b> <b>old</b>(<a href="DiemConfig.md#0x1_DiemConfig_spec_is_published">DiemConfig::spec_is_published</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;()) ==&gt;
        <b>global</b>&lt;<a href="DiemConfig.md#0x1_DiemConfig">DiemConfig</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;&gt;(@DiemRoot) ==
            <b>old</b>(<b>global</b>&lt;<a href="DiemConfig.md#0x1_DiemConfig">DiemConfig</a>&lt;<a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig">DiemConsensusConfig</a>&gt;&gt;(@DiemRoot));
}
</code></pre>




<pre><code><b>apply</b> <a href="DiemConsensusConfig.md#0x1_DiemConsensusConfig_DiemConsensusConfigRemainsSame">DiemConsensusConfigRemainsSame</a> <b>to</b> * <b>except</b> set;
</code></pre>

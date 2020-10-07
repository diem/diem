
<a name="0x1_LibraVersion"></a>

# Module `0x1::LibraVersion`

Maintains the version number for the Libra blockchain. The version is stored in a
LibraConfig, and may be updated by Libra root.


-  [Struct `LibraVersion`](#0x1_LibraVersion_LibraVersion)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_LibraVersion_initialize)
-  [Function `set`](#0x1_LibraVersion_set)
-  [Module Specification](#@Module_Specification_1)
    -  [Initialization](#@Initialization_2)
    -  [Access Control](#@Access_Control_3)
    -  [Other Invariants](#@Other_Invariants_4)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="LibraConfig.md#0x1_LibraConfig">0x1::LibraConfig</a>;
<b>use</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp">0x1::LibraTimestamp</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
</code></pre>



<a name="0x1_LibraVersion_LibraVersion"></a>

## Struct `LibraVersion`



<pre><code><b>struct</b> <a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>
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


<a name="0x1_LibraVersion_EINVALID_MAJOR_VERSION_NUMBER"></a>

Tried to set an invalid major version for the VM. Major versions must be strictly increasing


<pre><code><b>const</b> <a href="LibraVersion.md#0x1_LibraVersion_EINVALID_MAJOR_VERSION_NUMBER">EINVALID_MAJOR_VERSION_NUMBER</a>: u64 = 0;
</code></pre>



<a name="0x1_LibraVersion_initialize"></a>

## Function `initialize`

Publishes the LibraVersion config. Must be called during Genesis.


<pre><code><b>public</b> <b>fun</b> <a href="LibraVersion.md#0x1_LibraVersion_initialize">initialize</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraVersion.md#0x1_LibraVersion_initialize">initialize</a>(
    lr_account: &signer,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;(
        lr_account,
        <a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a> { major: 1 },
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the LibraRoot role [[H9]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigAbortsIf">LibraConfig::PublishNewConfigAbortsIf</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigEnsures">LibraConfig::PublishNewConfigEnsures</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;{payload: <a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a> { major: 1 }};
</code></pre>



</details>

<a name="0x1_LibraVersion_set"></a>

## Function `set`

Allows Libra root to update the major version to a larger version.


<pre><code><b>public</b> <b>fun</b> <a href="LibraVersion.md#0x1_LibraVersion_set">set</a>(lr_account: &signer, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraVersion.md#0x1_LibraVersion_set">set</a>(lr_account: &signer, major: u64) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_operating">LibraTimestamp::assert_operating</a>();

    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>let</b> old_config = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;();

    <b>assert</b>(
        old_config.major &lt; major,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraVersion.md#0x1_LibraVersion_EINVALID_MAJOR_VERSION_NUMBER">EINVALID_MAJOR_VERSION_NUMBER</a>)
    );

    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;(
        lr_account,
        <a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a> { major }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the LibraRoot role [[H9]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotOperating">LibraTimestamp::AbortsIfNotOperating</a>;
<b>aborts_if</b> <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;().major &gt;= major <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetAbortsIf">LibraConfig::SetAbortsIf</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;{account: lr_account};
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_SetEnsures">LibraConfig::SetEnsures</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;{payload: <a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a> { major }};
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="@Initialization_2"></a>

### Initialization


After genesis, version is published.


<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;();
</code></pre>



<a name="@Access_Control_3"></a>

### Access Control

Only "set" can modify the LibraVersion config [[H9]][PERMISSION]


<a name="0x1_LibraVersion_LibraVersionRemainsSame"></a>


<pre><code><b>schema</b> <a href="LibraVersion.md#0x1_LibraVersion_LibraVersionRemainsSame">LibraVersionRemainsSame</a> {
    <b>ensures</b> <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;()) ==&gt;
        <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) ==
            <b>old</b>(<b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()));
}
</code></pre>




<pre><code><b>apply</b> <a href="LibraVersion.md#0x1_LibraVersion_LibraVersionRemainsSame">LibraVersionRemainsSame</a> <b>to</b> * <b>except</b> set;
</code></pre>



The permission "UpdateLibraProtocolVersion" is granted to LibraRoot [[H9]][PERMISSION].


<pre><code><b>invariant</b> [<b>global</b>, isolated] <b>forall</b> addr: address <b>where</b> <b>exists</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;&gt;(addr):
    addr == <a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>();
</code></pre>



<a name="@Other_Invariants_4"></a>

### Other Invariants


Version number never decreases


<pre><code><b>invariant</b> <b>update</b> [<b>global</b>, isolated]
    <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;().major) &lt;= <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraVersion.md#0x1_LibraVersion">LibraVersion</a>&gt;().major;
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions

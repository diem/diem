
<a name="0x1_DiemTransactionPublishingOption"></a>

# Module `0x1::DiemTransactionPublishingOption`

This module defines a struct storing the publishing policies for the VM.


-  [Struct `DiemTransactionPublishingOption`](#0x1_DiemTransactionPublishingOption_DiemTransactionPublishingOption)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_DiemTransactionPublishingOption_initialize)
-  [Function `is_script_allowed`](#0x1_DiemTransactionPublishingOption_is_script_allowed)
-  [Function `is_module_allowed`](#0x1_DiemTransactionPublishingOption_is_module_allowed)
-  [Function `add_to_script_allow_list`](#0x1_DiemTransactionPublishingOption_add_to_script_allow_list)
-  [Function `set_open_script`](#0x1_DiemTransactionPublishingOption_set_open_script)
-  [Function `set_open_module`](#0x1_DiemTransactionPublishingOption_set_open_module)
-  [Module Specification](#@Module_Specification_1)
    -  [Initialization](#@Initialization_2)
    -  [Access Control](#@Access_Control_3)
    -  [Helper Functions](#@Helper_Functions_4)


<pre><code><b>use</b> <a href="DiemConfig.md#0x1_DiemConfig">0x1::DiemConfig</a>;
<b>use</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp">0x1::DiemTimestamp</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Roles.md#0x1_Roles">0x1::Roles</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_DiemTransactionPublishingOption"></a>

## Struct `DiemTransactionPublishingOption`

Defines and holds the publishing policies for the VM. There are three possible configurations:
1. No module publishing, only allow-listed scripts are allowed.
2. No module publishing, custom scripts are allowed.
3. Both module publishing and custom scripts are allowed.
We represent these as the following resource.


<pre><code><b>struct</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>script_allow_list: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>
 Only script hashes in the following list can be executed by the network. If the vector is empty, no
 limitation would be enforced.
</dd>
<dt>
<code>module_publishing_allowed: bool</code>
</dt>
<dd>
 Anyone can publish new module if this flag is set to true.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DiemTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT"></a>

The script hash already exists in the allowlist


<pre><code><b>const</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>: u64 = 1;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_EINVALID_SCRIPT_HASH"></a>

The script hash has an invalid length


<pre><code><b>const</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>: u64 = 0;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_SCRIPT_HASH_LENGTH"></a>



<pre><code><b>const</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x1_DiemTransactionPublishingOption_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_initialize">initialize</a>(dr_account: &signer, script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, module_publishing_allowed: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_initialize">initialize</a>(
    dr_account: &signer,
    script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    module_publishing_allowed: bool,
) {
    <a href="DiemTimestamp.md#0x1_DiemTimestamp_assert_genesis">DiemTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);

    <a href="DiemConfig.md#0x1_DiemConfig_publish_new_config">DiemConfig::publish_new_config</a>(
        dr_account,
        <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a> {
            script_allow_list, module_publishing_allowed
        }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_AbortsIfNotGenesis">DiemTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_PublishNewConfigAbortsIf">DiemConfig::PublishNewConfigAbortsIf</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_PublishNewConfigEnsures">DiemConfig::PublishNewConfigEnsures</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt; {
    payload: <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a> {
        script_allow_list, module_publishing_allowed
    }};
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_is_script_allowed"></a>

## Function `is_script_allowed`

Check if sender can execute script with <code>hash</code>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();

    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&publish_option.script_allow_list)
        || <a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, hash)
        || <a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a>;
</code></pre>




<a name="0x1_DiemTransactionPublishingOption_AbortsIfNoTransactionPublishingOption"></a>


<pre><code><b>schema</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a> {
    <b>include</b> <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_genesis">DiemTimestamp::is_genesis</a>() ==&gt; <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{};
}
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_is_module_allowed"></a>

## Function `is_module_allowed`

Check if a sender can publish a module


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool {
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a>;
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_add_to_script_allow_list"></a>

## Function `add_to_script_allow_list`

Add <code>new_hash</code> to the list of script hashes that is allowed to be executed by the network.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_add_to_script_allow_list">add_to_script_allow_list</a>(dr_account: &signer, new_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_add_to_script_allow_list">add_to_script_allow_list</a>(dr_account: &signer, new_hash: vector&lt;u8&gt;) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);

    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_hash) == <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>));

    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();
    <b>if</b> (<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, &new_hash)) {
          <b>abort</b> <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>)
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> publish_option.script_allow_list, new_hash);

    <a href="DiemConfig.md#0x1_DiemConfig_set">DiemConfig::set</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;(dr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<a name="0x1_DiemTransactionPublishingOption_allow_list$8"></a>
<b>let</b> allow_list = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;().script_allow_list;
<b>aborts_if</b> <a href="Vector.md#0x1_Vector_length">Vector::length</a>(new_hash) != <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a> <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>aborts_if</b> <a href="Vector.md#0x1_Vector_spec_contains">Vector::spec_contains</a>(allow_list, new_hash) <b>with</b> <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetAbortsIf">DiemConfig::SetAbortsIf</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{account: dr_account};
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_set_open_script"></a>

## Function `set_open_script`

Allow the execution of arbitrary script or not.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_set_open_script">set_open_script</a>(dr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_set_open_script">set_open_script</a>(dr_account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();

    publish_option.script_allow_list = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <a href="DiemConfig.md#0x1_DiemConfig_set">DiemConfig::set</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;(dr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetAbortsIf">DiemConfig::SetAbortsIf</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{account: dr_account};
</code></pre>



</details>

<a name="0x1_DiemTransactionPublishingOption_set_open_module"></a>

## Function `set_open_module`

Allow module publishing from arbitrary sender or not.


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_set_open_module">set_open_module</a>(dr_account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_set_open_module">set_open_module</a>(dr_account: &signer, open_module: bool) {
    <a href="Roles.md#0x1_Roles_assert_diem_root">Roles::assert_diem_root</a>(dr_account);

    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_get">DiemConfig::get</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed = open_module;
    <a href="DiemConfig.md#0x1_DiemConfig_set">DiemConfig::set</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;(dr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotDiemRoot">Roles::AbortsIfNotDiemRoot</a>{account: dr_account};
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_AbortsIfNotPublished">DiemConfig::AbortsIfNotPublished</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="DiemConfig.md#0x1_DiemConfig_SetAbortsIf">DiemConfig::SetAbortsIf</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;{account: dr_account};
</code></pre>



</details>

<a name="@Module_Specification_1"></a>

## Module Specification



<a name="@Initialization_2"></a>

### Initialization



<pre><code><b>invariant</b> [<b>global</b>] <a href="DiemTimestamp.md#0x1_DiemTimestamp_is_operating">DiemTimestamp::is_operating</a>() ==&gt;
    <a href="DiemConfig.md#0x1_DiemConfig_spec_is_published">DiemConfig::spec_is_published</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();
</code></pre>



<a name="@Access_Control_3"></a>

### Access Control

Only <code>add_to_script_allow_list</code>, <code>set_open_script</code>, and <code>set_open_module</code> can modify the
DiemTransactionPublishingOption config [[H11]][PERMISSION]


<a name="0x1_DiemTransactionPublishingOption_DiemVersionRemainsSame"></a>


<pre><code><b>schema</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_DiemVersionRemainsSame">DiemVersionRemainsSame</a> {
    <b>ensures</b> <b>old</b>(<a href="DiemConfig.md#0x1_DiemConfig_spec_is_published">DiemConfig::spec_is_published</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;()) ==&gt;
        <b>global</b>&lt;<a href="DiemConfig.md#0x1_DiemConfig">DiemConfig</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()) ==
            <b>old</b>(<b>global</b>&lt;<a href="DiemConfig.md#0x1_DiemConfig">DiemConfig</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_DIEM_ROOT_ADDRESS">CoreAddresses::DIEM_ROOT_ADDRESS</a>()));
}
</code></pre>




<pre><code><b>apply</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_DiemVersionRemainsSame">DiemVersionRemainsSame</a> <b>to</b> * <b>except</b> add_to_script_allow_list, set_open_script, set_open_module;
</code></pre>



<a name="@Helper_Functions_4"></a>

### Helper Functions



<a name="0x1_DiemTransactionPublishingOption_spec_is_script_allowed"></a>


<pre><code><b>define</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_spec_is_script_allowed">spec_is_script_allowed</a>(account: signer, hash: vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_spec_get_config">DiemConfig::spec_get_config</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();
    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(publish_option.script_allow_list)
        || <a href="Vector.md#0x1_Vector_spec_contains">Vector::spec_contains</a>(publish_option.script_allow_list, hash)
        || <a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account)
}
<a name="0x1_DiemTransactionPublishingOption_spec_is_module_allowed"></a>
<b>define</b> <a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption_spec_is_module_allowed">spec_is_module_allowed</a>(account: signer): bool {
    <b>let</b> publish_option = <a href="DiemConfig.md#0x1_DiemConfig_spec_get_config">DiemConfig::spec_get_config</a>&lt;<a href="DiemTransactionPublishingOption.md#0x1_DiemTransactionPublishingOption">DiemTransactionPublishingOption</a>&gt;();
    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_diem_root_role">Roles::has_diem_root_role</a>(account)
}
</code></pre>


[//]: # ("File containing references which can be used from documentation")
[ACCESS_CONTROL]: https://github.com/diem/dip/blob/master/dips/dip-2.md
[ROLE]: https://github.com/diem/dip/blob/master/dips/dip-2.md#roles
[PERMISSION]: https://github.com/diem/dip/blob/master/dips/dip-2.md#permissions

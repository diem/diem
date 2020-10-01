
<a name="0x1_LibraTransactionPublishingOption"></a>

# Module `0x1::LibraTransactionPublishingOption`



-  [Struct <code><a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a></code>](#0x1_LibraTransactionPublishingOption_LibraTransactionPublishingOption)
-  [Const <code><a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a></code>](#0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH)
-  [Const <code><a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a></code>](#0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH)
-  [Const <code><a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a></code>](#0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT)
-  [Function <code>initialize</code>](#0x1_LibraTransactionPublishingOption_initialize)
-  [Function <code>is_script_allowed</code>](#0x1_LibraTransactionPublishingOption_is_script_allowed)
-  [Function <code>is_module_allowed</code>](#0x1_LibraTransactionPublishingOption_is_module_allowed)
-  [Function <code>add_to_script_allow_list</code>](#0x1_LibraTransactionPublishingOption_add_to_script_allow_list)
-  [Function <code>set_open_script</code>](#0x1_LibraTransactionPublishingOption_set_open_script)
-  [Function <code>set_open_module</code>](#0x1_LibraTransactionPublishingOption_set_open_module)


<a name="0x1_LibraTransactionPublishingOption_LibraTransactionPublishingOption"></a>

## Struct `LibraTransactionPublishingOption`

Defines and holds the publishing policies for the VM. There are three possible configurations:
1. No module publishing, only allowlisted scripts are allowed.
2. No module publishing, custom scripts are allowed.
3. Both module publishing and custom scripts are allowed.
We represent these as the following resource.


<pre><code><b>struct</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>script_allow_list: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>module_publishing_allowed: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH"></a>

## Const `SCRIPT_HASH_LENGTH`



<pre><code><b>const</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH"></a>

## Const `EINVALID_SCRIPT_HASH`

The script hash has an invalid length


<pre><code><b>const</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>: u64 = 0;
</code></pre>



<a name="0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT"></a>

## Const `EALLOWLIST_ALREADY_CONTAINS_SCRIPT`

The script hash already exists in the allowlist


<pre><code><b>const</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>: u64 = 1;
</code></pre>



<a name="0x1_LibraTransactionPublishingOption_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_initialize">initialize</a>(lr_account: &signer, script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, module_publishing_allowed: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_initialize">initialize</a>(
    lr_account: &signer,
    script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    module_publishing_allowed: bool,
) {
    <a href="LibraTimestamp.md#0x1_LibraTimestamp_assert_genesis">LibraTimestamp::assert_genesis</a>();
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <a href="LibraConfig.md#0x1_LibraConfig_publish_new_config">LibraConfig::publish_new_config</a>(
        lr_account,
        <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a> {
            script_allow_list, module_publishing_allowed
        }
    );
}
</code></pre>



</details>

<details>
<summary>Specification</summary>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_AbortsIfNotGenesis">LibraTimestamp::AbortsIfNotGenesis</a>;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigAbortsIf">LibraConfig::PublishNewConfigAbortsIf</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;;
<b>include</b> <a href="LibraConfig.md#0x1_LibraConfig_PublishNewConfigEnsures">LibraConfig::PublishNewConfigEnsures</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt; {
    payload: <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a> {
        script_allow_list, module_publishing_allowed
    }};
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_is_script_allowed"></a>

## Function `is_script_allowed`



<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&publish_option.script_allow_list)
        || <a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, hash)
        || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a>;
</code></pre>




<a name="0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption"></a>


<pre><code><b>schema</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a> {
    <b>include</b> <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_genesis">LibraTimestamp::is_genesis</a>() ==&gt; <a href="LibraConfig.md#0x1_LibraConfig_AbortsIfNotPublished">LibraConfig::AbortsIfNotPublished</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;{};
}
</code></pre>




<pre><code><b>invariant</b> [<b>global</b>] <a href="LibraTimestamp.md#0x1_LibraTimestamp_is_operating">LibraTimestamp::is_operating</a>() ==&gt;
    <a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_is_module_allowed"></a>

## Function `is_module_allowed`



<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>include</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_AbortsIfNoTransactionPublishingOption">AbortsIfNoTransactionPublishingOption</a>;
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_add_to_script_allow_list"></a>

## Function `add_to_script_allow_list`



<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, new_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_add_to_script_allow_list">add_to_script_allow_list</a>(lr_account: &signer, new_hash: vector&lt;u8&gt;) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_hash) == <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>));

    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
    <b>if</b> (<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, &new_hash)) {
          <b>abort</b> <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>)
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> publish_option.script_allow_list, new_hash);

    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
</code></pre>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>aborts_with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>, <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>, <a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>, <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_set_open_script"></a>

## Function `set_open_script`



<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_set_open_script">set_open_script</a>(lr_account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_set_open_script">set_open_script</a>(lr_account: &signer) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.script_allow_list = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
</code></pre>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>aborts_with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>, <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>, <a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>, <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>



</details>

<a name="0x1_LibraTransactionPublishingOption_set_open_module"></a>

## Function `set_open_module`



<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_set_open_module">set_open_module</a>(lr_account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_set_open_module">set_open_module</a>(lr_account: &signer, open_module: bool) {
    <a href="Roles.md#0x1_Roles_assert_libra_root">Roles::assert_libra_root</a>(lr_account);

    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_get">LibraConfig::get</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();

    publish_option.module_publishing_allowed = open_module;
    <a href="LibraConfig.md#0x1_LibraConfig_set">LibraConfig::set</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;(lr_account, publish_option);
}
</code></pre>



</details>

<details>
<summary>Specification</summary>



<pre><code><b>pragma</b> aborts_if_is_partial = <b>true</b>;
</code></pre>


Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].


<pre><code><b>include</b> <a href="Roles.md#0x1_Roles_AbortsIfNotLibraRoot">Roles::AbortsIfNotLibraRoot</a>{account: lr_account};
<b>aborts_with</b> <a href="Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a>, <a href="Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a>, <a href="Errors.md#0x1_Errors_REQUIRES_CAPABILITY">Errors::REQUIRES_CAPABILITY</a>, <a href="Errors.md#0x1_Errors_NOT_PUBLISHED">Errors::NOT_PUBLISHED</a>;
</code></pre>


Only add_to_script_allow_list, set_open_script, and set_open_module can modify the
LibraTransactionPublishingOption config [[H10]][PERMISSION]


<a name="0x1_LibraTransactionPublishingOption_LibraVersionRemainsSame"></a>


<pre><code><b>schema</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_LibraVersionRemainsSame">LibraVersionRemainsSame</a> {
    <b>ensures</b> <b>old</b>(<a href="LibraConfig.md#0x1_LibraConfig_spec_is_published">LibraConfig::spec_is_published</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;()) ==&gt;
        <b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()) ==
            <b>old</b>(<b>global</b>&lt;<a href="LibraConfig.md#0x1_LibraConfig">LibraConfig</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_LIBRA_ROOT_ADDRESS">CoreAddresses::LIBRA_ROOT_ADDRESS</a>()));
}
</code></pre>




<pre><code><b>apply</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_LibraVersionRemainsSame">LibraVersionRemainsSame</a> <b>to</b> * <b>except</b> add_to_script_allow_list, set_open_script, set_open_module;
</code></pre>




<a name="0x1_LibraTransactionPublishingOption_spec_is_script_allowed"></a>


<pre><code><b>define</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_spec_is_script_allowed">spec_is_script_allowed</a>(account: signer, hash: vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_spec_get_config">LibraConfig::spec_get_config</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(publish_option.script_allow_list)
        || <a href="Vector.md#0x1_Vector_spec_contains">Vector::spec_contains</a>(publish_option.script_allow_list, hash)
        || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
<a name="0x1_LibraTransactionPublishingOption_spec_is_module_allowed"></a>
<b>define</b> <a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption_spec_is_module_allowed">spec_is_module_allowed</a>(account: signer): bool {
    <b>let</b> publish_option = <a href="LibraConfig.md#0x1_LibraConfig_spec_get_config">LibraConfig::spec_get_config</a>&lt;<a href="LibraTransactionPublishingOption.md#0x1_LibraTransactionPublishingOption">LibraTransactionPublishingOption</a>&gt;();
    publish_option.module_publishing_allowed || <a href="Roles.md#0x1_Roles_has_libra_root_role">Roles::has_libra_root_role</a>(account)
}
</code></pre>



</details>
[ROLE]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#roles
[PERMISSION]: https://github.com/libra/libra/blob/master/language/move-prover/doc/user/access-control.md#permissions

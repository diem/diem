
<a name="0x1_CoreErrors"></a>

# Module `0x1::CoreErrors`

### Table of Contents

-  [Function `NOT_GENESIS`](#0x1_CoreErrors_NOT_GENESIS)
    -  [General Error Codes](#0x1_CoreErrors_@General_Error_Codes)
-  [Function `NOT_OPERATING`](#0x1_CoreErrors_NOT_OPERATING)
-  [Function `NOT_INITIALIZED`](#0x1_CoreErrors_NOT_INITIALIZED)
-  [Function `ALREADY_INITIALIZED`](#0x1_CoreErrors_ALREADY_INITIALIZED)
-  [Function `INVALID_INPUT`](#0x1_CoreErrors_INVALID_INPUT)
-  [Function `OUT_OF_RANGE`](#0x1_CoreErrors_OUT_OF_RANGE)
-  [Function `INTERNAL`](#0x1_CoreErrors_INTERNAL)
-  [Function `NOT_LIBRA_ROOT_ADDRESS`](#0x1_CoreErrors_NOT_LIBRA_ROOT_ADDRESS)
    -  [Errors Codes Related to Expected Signers](#0x1_CoreErrors_@Errors_Codes_Related_to_Expected_Signers)
-  [Function `NOT_VM_RESERVED_ADDRESS`](#0x1_CoreErrors_NOT_VM_RESERVED_ADDRESS)
-  [Function `NOT_LIBRA_ROOT_ROLE`](#0x1_CoreErrors_NOT_LIBRA_ROOT_ROLE)
    -  [Error Codes Related to Expected Roles](#0x1_CoreErrors_@Error_Codes_Related_to_Expected_Roles)
-  [Function `NOT_TREASURY_COMPLIANCE_ROLE`](#0x1_CoreErrors_NOT_TREASURY_COMPLIANCE_ROLE)
-  [Function `FIRST_AVAILABLE_CUSTOM_CODE`](#0x1_CoreErrors_FIRST_AVAILABLE_CUSTOM_CODE)
    -  [Custom Error Codes](#0x1_CoreErrors_@Custom_Error_Codes)

Module containing common error codes used in the framework.


<a name="0x1_CoreErrors_NOT_GENESIS"></a>

## Function `NOT_GENESIS`


<a name="0x1_CoreErrors_@General_Error_Codes"></a>

### General Error Codes

Operation only allowed in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_GENESIS">NOT_GENESIS</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_GENESIS">NOT_GENESIS</a>(): u64 { 101 }
</code></pre>



</details>

<a name="0x1_CoreErrors_NOT_OPERATING"></a>

## Function `NOT_OPERATING`

Operation only allowed outside of genesis.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_OPERATING">NOT_OPERATING</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_OPERATING">NOT_OPERATING</a>(): u64 { 102 }
</code></pre>



</details>

<a name="0x1_CoreErrors_NOT_INITIALIZED"></a>

## Function `NOT_INITIALIZED`

A resource is attempted to publish twice.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_INITIALIZED">NOT_INITIALIZED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_INITIALIZED">NOT_INITIALIZED</a>(): u64 { 103 }
</code></pre>



</details>

<a name="0x1_CoreErrors_ALREADY_INITIALIZED"></a>

## Function `ALREADY_INITIALIZED`

A resource is attempted to publish twice.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_ALREADY_INITIALIZED">ALREADY_INITIALIZED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_ALREADY_INITIALIZED">ALREADY_INITIALIZED</a>(): u64 { 104 }
</code></pre>



</details>

<a name="0x1_CoreErrors_INVALID_INPUT"></a>

## Function `INVALID_INPUT`

Invalid input data has been provided.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_INVALID_INPUT">INVALID_INPUT</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_INVALID_INPUT">INVALID_INPUT</a>(): u64 { 105 }
</code></pre>



</details>

<a name="0x1_CoreErrors_OUT_OF_RANGE"></a>

## Function `OUT_OF_RANGE`

An input parameter is out of range.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_OUT_OF_RANGE">OUT_OF_RANGE</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_OUT_OF_RANGE">OUT_OF_RANGE</a>(): u64 { 106 }
</code></pre>



</details>

<a name="0x1_CoreErrors_INTERNAL"></a>

## Function `INTERNAL`

An internal error has happened.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_INTERNAL">INTERNAL</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_INTERNAL">INTERNAL</a>(): u64 { 107 }
</code></pre>



</details>

<a name="0x1_CoreErrors_NOT_LIBRA_ROOT_ADDRESS"></a>

## Function `NOT_LIBRA_ROOT_ADDRESS`


<a name="0x1_CoreErrors_@Errors_Codes_Related_to_Expected_Signers"></a>

### Errors Codes Related to Expected Signers

Signer expected to be the singleton root.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_LIBRA_ROOT_ADDRESS">NOT_LIBRA_ROOT_ADDRESS</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_LIBRA_ROOT_ADDRESS">NOT_LIBRA_ROOT_ADDRESS</a>(): u64 { 201 }
</code></pre>



</details>

<a name="0x1_CoreErrors_NOT_VM_RESERVED_ADDRESS"></a>

## Function `NOT_VM_RESERVED_ADDRESS`

Signer expected to be the VM.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_VM_RESERVED_ADDRESS">NOT_VM_RESERVED_ADDRESS</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_VM_RESERVED_ADDRESS">NOT_VM_RESERVED_ADDRESS</a>(): u64 { 202 }
</code></pre>



</details>

<a name="0x1_CoreErrors_NOT_LIBRA_ROOT_ROLE"></a>

## Function `NOT_LIBRA_ROOT_ROLE`


<a name="0x1_CoreErrors_@Error_Codes_Related_to_Expected_Roles"></a>

### Error Codes Related to Expected Roles

Signer expected to have root role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_LIBRA_ROOT_ROLE">NOT_LIBRA_ROOT_ROLE</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_LIBRA_ROOT_ROLE">NOT_LIBRA_ROOT_ROLE</a>(): u64 { 301 }
</code></pre>



</details>

<a name="0x1_CoreErrors_NOT_TREASURY_COMPLIANCE_ROLE"></a>

## Function `NOT_TREASURY_COMPLIANCE_ROLE`

Signer expected to have treasury compliance role.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_TREASURY_COMPLIANCE_ROLE">NOT_TREASURY_COMPLIANCE_ROLE</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_NOT_TREASURY_COMPLIANCE_ROLE">NOT_TREASURY_COMPLIANCE_ROLE</a>(): u64 { 302 }
</code></pre>



</details>

<a name="0x1_CoreErrors_FIRST_AVAILABLE_CUSTOM_CODE"></a>

## Function `FIRST_AVAILABLE_CUSTOM_CODE`


<a name="0x1_CoreErrors_@Custom_Error_Codes"></a>

### Custom Error Codes

Custom error codes can be defined on a per-module basis. Each custom error code should provide a public
function to make this code available for scripts and other modules, using naming conventions as here.
Custom error codes should start at the value below. They must not overlap with custom error
codes defined in other modules (please check existing codes when introducing a new one to achieve this.)
Also consider that your custom error codes must be made known to applications running on top of Libra
so they can be converted into appropriate user messages, as these codes become part of the public APIs.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_FIRST_AVAILABLE_CUSTOM_CODE">FIRST_AVAILABLE_CUSTOM_CODE</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_CoreErrors_FIRST_AVAILABLE_CUSTOM_CODE">FIRST_AVAILABLE_CUSTOM_CODE</a>(): u64 { 100000 }
</code></pre>



</details>

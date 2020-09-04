---
author: Calibra Engineering
title: Simplifying Libra Transaction Payloads: Deprecation of the "Program" Type
---
<script>
    let items = document.getElementsByClassName("post-meta");   
    for (var i = items.length - 1; i >= 0; i--) {
        if (items[i].innerHTML = '<p class="post-meta">October 22, 2019</p>') items[i].innerHTML = '<p class="post-meta">October 22, 2019</p>';
    }
    var slug = location.pathname.slice(location.pathname.lastIndexOf('/')+1);
    var redirect = 'https://libra.org/en-US/blog/' + slug;
    window.location = redirect;    
</script>


## Overview

We are simplifying the types of payload a Libra transaction can contain. Currently, the transaction payload of type &quot;Program&quot; allows both a Move module to be published and a Move transaction script to be executed within the same transaction. We will soon be deprecating the &quot;Program&quot; type. The benefits of making this change and the impact of this change are described in this document.

## Payload of a Libra Transaction

Currently, the [payload of a Libra transaction](https://github.com/libra/libra/blob/1f04143cb2490294ad4401ab73822d80260c4262/types/src/transaction.rs#L273-L281) has the following three variants:
```
pub enumTransactionPayload {
    /// A regular programmatic transaction that is executed by the VM.
    Program(Program),     /// A genesis transaction.
    WriteSet(WriteSet),
    /// A transaction that publishes a module.
    Module(Module),
    /// A transaction that executes a script.
    Script(Script),
}
```

- **Program**
  - Encodes a peer-to-peer transaction.
  - Within the same transaction, Move modules can be published and Move transaction scripts can be executed.
- **Writeset**
  - Contains a simple writeset.
  - Used for genesis transactions only.
- **Module**
  - Used to publish a Move module.
  - The payload is a single serialized Move [CompiledModule](https://github.com/libra/libra/blob/1f04143cb2490294ad4401ab73822d80260c4262/language/vm/src/file_format.rs#L1390).
  - A transaction with this type of payload just publishes the CompiledModule and does nothing else.
- **Script**
  - Used to execute a Move transaction script.
  - The payload is a serialized Move [CompiledScript](https://github.com/libra/libra/blob/1f04143cb2490294ad4401ab73822d80260c4262/language/vm/src/file_format.rs#L1292).
  - A transaction with this type of payload just executes the CompiledScript and does nothing else.

**Note:** Currently, module transactions are accepted on local networks only.

## What Will Change?

You may have noticed that the &quot;Program&quot; type is essentially a combination of the &quot;Module&quot; and the &quot;Script&quot; types. In the next week, the &quot;Program&quot; type will be removed from this implementation.

## What Are the Benefits of This Change?

There are two major benefits to using &quot;Script&quot; and &quot;Module&quot; types separately instead of  combining them into the (soon-to-be deprecated) &quot;Program&quot; type:

- **Reduces the size of a transaction** - In the current implementation, for a regular peer-to-peer transaction, the module information should be encoded in the transaction as an empty &quot;to-be-published&quot; modules vector, even if there is no module to be published. This costs extra bytes.
- **Simplifies the VM structure** - The VM caches the modules published on-chain for better performance. The semantics for transactions of payload type &quot;Program&quot; complicates the caching story of VM significantly. The VM must deal with the fact that, even if a &quot;Program&quot; attempts to publish a legitimate module, it can fail subsequently during the execution of the transaction script. When there&#39;s a script execution failure, the VM needs to get rid of  the cached module so that this specific module is not visible to future transactions.

## What Do We Lose From This Change?

Nothing!

Move modules are stateless, and there&#39;s no difference between publishing modules within one transaction or across different transactions.

## How Do We Transition to the New Implementation?

The Libra codebase has been modified and all occurrences of the old transaction type have been removed [(PR #923)](https://github.com/libra/libra/pull/923).

If you want to execute a transaction with a &quot;Script&quot; payload, the current transaction payload would look like this:
```
TransactionPayload::Program(
Program::new(
/*code*/ some_bytes,
/*args*/ args_vector,
/*module*/ vec![])
)
```

You will need to change it to:

```
TransactionPayload::Script(
Script::new(
/*code*/ some_bytes,
/*args*/ args_vector)
)
```

Similarly, if you were trying  to publish a module, your current transaction payload would look like this:
```
TransactionPayload::Program(
Program::new(
/*code*/ some_bytes,    // Just an empty main function
/*args*/ args_vector,
/*module*/ vec![compiled_module_bytes])
)
```

You will need to change it to:
```
TransactionPayload::Module(Module::new(/*code*/ compiled_module_bytes))
```

We will support all transaction payload types on testnet for a few more weeks, following which the &quot;Program&quot; variant in transaction.rs will be removed.

## Who is Impacted by This Change?

As the transaction payload bytes remain unchanged, this transition will be easy. However, wallet and client developers should be aware of this transition, as this will change the format of the transaction accepted by the Libra testnet.

## Libra Developer Documentation Updated

The Libra developer documentation has been updated to reflect these new changes. Assume that any Libra transaction contains a script payload.

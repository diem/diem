# Friend Visibility

* Status: Implemented in Move 1.2

## Introduction

Friend visibility is a new feature in Move version 1.2 to give more control about where a function can be used. Previously a function had either public or private visibility, where a public function is accessible anywhere but a private function can be called only from within the module where it is defined. Friend-visible functions can be called only from explicitly allowed modules.

## Motivations

### Overly Permissive Function Visibility Model

The simple public/private visibility scheme required using public visibility for “limited-access” functions, i.e., functions that:

* are designed to have limited access by a known and specific set of modules only (i.e., an allowlist), and
* should not be accessed by any other modules outside this specific allowlist.

Take all the `initialize` functions in the Diem framework as an example. In theory, the `initialize` functions should only be used by the `Genesis` module and never be exposed to any other modules nor scripts. However, due to the limitation in the current visibility model, these `initialize` functions have to be made `public` (in order for the `Genesis` module to call them) and both runtime capability checks and static verification are enforced to make sure that these functions will abort if not called from a genesis state.

### Inflexibility in Future Module Updates

Public functions have very restricted updating rules: a public function can never be deleted or renamed, and its function signature can never be altered. The rationale behind this restriction is that a public function is a contract made to the whole world and once the contract is made, the API should not be easily altered, as altering the API might break code that tries to invoke the public function. The owner of the public function has no control on who can invoke the function. In fact, knowing all the call sites requires a global scan of all code published in storage, which is not always possible nor scalable in a blockchain network with an open-publishing model.

In contrast, a friend function is only a contract made to the friends of a module, and furthermore, the module owner controls the membership of the friend list. That is, the module owner has complete knowledge of which modules may access a friend function. As a result, it is much easier to update a friend function because only the modules in the friend list needs to be coordinated for the change. This is especially true when a friend function and all its friend modules are maintained by the same owner.

### Simplification Opportunities for Specification and Verification

Friend visibility can help simplify specification writing and verification with the Move prover. For example, given a friend function and its host module’s friend list, we can easily and exhaustively find all callsites of the friend function. With this information, we could, as an option, completely skip the specification of a friend function and inline the implementation into the caller. This may lead to further simplification in the verification techniques and allows stronger properties to be proved. In contrast, for a public function, it is necessary to write complete and accurate specifications for the function.

## Description

Friend visibility expands the set of possible visibility levels:

* private (no modifier)
* `public(friend)`
* `public(script)`, and
* `public`.

These respectively correspond to `Private`, `Friend`, `Script`, and `Public` in the Move bytecode file format. Script visibility addresses an orthogonal problem in the Diem Framework and more details can be found in the [Script Visibility](2-script-visibility.md) change description.

Beside the new `public(friend)` modifier, each module is allowed to have a friend list, specifiable as zero or more
`friend <address::name>` statements that list the modules trusted by the host module. Modules in the friend list are permitted to call a `public(friend)` function defined in the host module, but a non-friend module is prohibited from accessing a `public(friend)` function.

### New Visibility Modifier

`public(friend)` is a new visibility modifier that can be applied to any function definition in a module. A `public(friend)` function can be invoked by any other functions in the same module (say module `M`), or any function defined in one of the modules in the friend list of module `M`.

Besides this visibility rule, `public(friend)` functions follow the same rules as any other module function, meaning they can invoke other functions in the same module (except `public(script)` functions), create new struct instances, access the global storage (of types declared in that module), etc.

### Friend List Declarations

A module can declare other modules as friends via friend declaration statements, in the format of

* `friend <address::name>` — friend declaration using fully qualified  module name
* `friend <module-name-alias>` — friend declaration using a module name alias, where the module alias is introduced via the `use` statement.

A module may have multiple friend declarations, and the union of all the friend modules is recorded in the friend list, which is a new section in the bytecode file format. For readability, friend declarations should generally be placed near the beginning of the module definition. Note that Move scripts cannot declare friend modules as the concept of friend functions does not even exist in scripts.

Friend declarations are subject to the following rules:

* A module cannot declare itself as a friend.
    * e.g., `0x2::M` cannot declare `0x2::M` as a friend.
* Friend modules must be within the same account address.
    * e.g., `0x2::M` cannot declare `0x3::N` as a friend.
    * Note: this is not a technical requirement but rather a policy decision which may be relaxed later.
* Friends relationships cannot create cyclic module dependencies.
    * Cycles are not allowed in the friend relationships. E.g., `0x2::A` friends `0x2::B` friends `0x2::C` friends `0x2::A` is not allowed.
    * More generally, declaring a friend module adds a dependency upon the current module to the friend module (because the purpose is for the friend to call functions in the current module). If that friend module is already used, either directly or transitively, a cycle of dependencies would be created. E.g., a cycle would be created if `0x2::A` friends `0x2::B` and `0x2::A` also calls a function `0x2::B::foo().`
* Friends must exist when the module is published.
    * e.g., `0x2::M` cannot declare `0x2::X` as a friend if `0x2::X` cannot be resolved by the loader.
* The friend list for a module cannot contain duplicates.

## Examples

A typical module with `public(friend)` functions and its friend modules is shown in the following example:

```
address 0x2 {
  module A {
    // friend declaration via fully qualified module name
    friend 0x2::B;

    // friend declaration via module alias
    use 0x2::C;
    friend C;

    public(friend) fun foo() {
      // a friend function can call other non-script functions in the same module
      i_am_private();
      i_am_public();
      bar();
    }
    public(friend) fun bar() {}

    fun i_am_private() {
      // other functions in the same module can also call friend functions
      bar();
    }
    public fun i_am_public() {
      // other functions in the same module can also call friend functions
      bar();
    }
  }

  module B {
    use 0x2::A;

    public fun foo() {
      // as a friend of 0x2::A, functions in B can call friend functions in A
      A::foo();
    }

    public fun bar() {
      0x2::A::bar();
    }
  }
}
```

## Alternatives

### Granularity of the Friend List

* Module-to-Module (adopted)
    * Module `B` is a friend of module `A` — any function in module `B` can access any friend function in module `A`
    * Modules have been the trust boundary in Move language, as evidenced by:
        * the existing visibility model where public and private are defined with regard to the hosting module of a particular function;
        * the design of Struct / Resource type where only the module that defines the Struct / Resource may access the internals of the type.
    * Therefore, it is more natural for modules to be the trust boundary of friend function accesses as well.
    * Another reason is that it resonates with the granularity of the friend feature found in other languages (e.g., C++).
* Module-to-Function
    * Module `B` is a friend of function `foo()` — any function in module `B` can call the friend function `foo()`
    * This is a more fine-grained version of Module-to-Module friendship declaration and is also found in other languages (e.g., C++ also supports Module-to-Function friendship). The reasons we did not choose this option are mostly 1) it breaks the mental model that modules are the boundaries in Move, and 2) it may lead to the case where a module (e.g., module `A`) is a friend of every friend function and `friend A` needs to be specified repeatedly for each friend function.
* Function-to-Module
    * Function `foo()` is a friend of module `A` — function `foo()` can call any friend function in module `A`
    * The reason this option is not chosen is because it seems weird to express, as a developer, that we trust function `0x3::B::foo()` but not function `0x3::B::bar()`, especially given that both `bar()` and `foo()` reside in the same module by `0x3::B`. We could not contemplate a valid use case for this scenario.
* Function-to-Function
    * Function `foo()` is a friend of function `bar()` — function `foo()` can call friend function `bar()`
    * Besides the reason that it feels weird trusting one function in a module but not another (similar to the Function-to-Module option), we feel that this scheme is too fine-grained and would cause inflexibility in development, especially on function name updates. To illustrate, suppose `foo()` is a private function in module `B`, and `bar()` is a friend function in module `A`. This scheme requires that when the private function `foo` is renamed, something in module A needs to be updated as well! Function `foo()` is no longer “private” to module `B` under this scheme.

### Location of Friend Declarations

* Callee-side declaration (adopted)
    * The code owner who develops the module is responsible for specifying who can be a friend of this module at the time of writing the source code. If later the developers want to add / remove new friends, they can always update the friend list and re-publish the module on-chain (subject to updatability and compatibility checking).
    * This is the most natural way of defining the friend list, since the friend list is embedded the same source file as the module source code. Compared with the alternative — caller-side declaration — it is cognitively easier for developers to figure out who may interact with the friend functions and how the friend functions should be hardened by looking within the same file.
* Caller-side declaration
    * An alternative thinking is to have the user of a friend function to “request” friendship permission, instead of having the owner of the friend function to “grant” friendship. To illustrate, if module `B` wants to access some friend functions in module `A`, then, the friendship with module `A` will be declared in the source code of module `B` (while the callee-side declaration requires that the friendship is declared in the source code of module `A`).
    * A major drawback in this alternative is that the code owner does not have a list of friend relationships if the developers do not actively maintain one. For friend relationships, the source of truth is likely to be stored on-chain either via 1) a VM-updatable section in on-chain module bytecode, or 2) a new `FriendList` entity in users’ accounts. More importantly, by looking at the source code of a module, the developers have no clue who can access the friend function and how the interaction may happen.

### Publishing Order

Cross-module references complicate the process for publishing those modules. The issue can be illustrated in the following:

```
address 0x2 {
  module M {
    friend 0x2::N;
    public(friend) fun foo() {}
  }
  module N {
    use 0x2::M;
    fun bar() { M::foo(); }
  }
}
```

Suppose we define two modules `M` and `N` like the above:

* Module `N` depends on `M` because `N` contains `use 0x2::M`
* But, at the same time, module `M` refers to `N` because `M` specifies `friend 0x2::N`

Now think about how we should publish them on-chain....

* With the current one-module-at-a-time publishing model:
    * Obviously module `M` has to be published first. Otherwise, publishing module `N` first will make `N::bar()` fail miserably, while publishing `M` first should have no bad effects because no one can call `M::foo()` anyway.
    * But, when publishing `M`, the bytecode verifier sees the visibility constraint, and it is a pointer to a nonexistent function `N::bar()`. The bytecode verifier will *not* try to resolve this function handle. It *must* tolerate that this visibility constraint is a forward declaration.
    * A risk associated with the above procedure is the possibility of a race condition when publishing module `N`. Suppose both Alice and Eve can publish to `0x2`. When `M` is published, both Alice and Eve see that `M` declares `N` as a friend. Eve may race against Alice to publish module `N` first, with a bad `bar()` function, to exploit the trust the developer of module `M` placed on Alice.
    * A solution to this problem is enforcing a safer but more convoluted module publication flow that still uses the one-module-at-a-time publishing model. For the example above, the flow would require three steps:
        * publish an empty module `N` as a place holder
        * publish module `M`
        * publish the updated module `N` that uses the friend function `M`
* With a future multi-signer, multi-module publication model:
    * To avoid a convoluted module publishing flow, another solution is to use a multi-signer + multi-module publication model that allows a bundle of modules to be published / updated atomically, even if these modules reside in different user accounts. In the above case, if we could publish `M` and `N` atomically in one transaction, there would be no risk of a race condition and there is no need to go through the three-step module publishing flow.

### Other Schemes for “Shared” Visibility

* Address visibility
    * Java uses a “package” concept that maps to a given location (namespace) which could roughly be thought of as an address (the address the modules are published under). Addresses in Move could serve the role of “package” in Java. With that approach, we could do something like `public(address)` — or just `internal` — that would allow for cross module visibility, but just under that address. The owner of that address would control all publishing into that address. This kind of visibility would be easy to enforce by a verifier given the address constraints. That is, the target function when linking would have to be in a module published under the same address.
    * The problem with this model is that we have no way to control subsequent publishing into that address, which could violate the principle in Move that all bindings are known when one publishes and they cannot be altered. If publishing under the same address gave someone access to the internal state of other modules it would be possible to read and alter state that was originally intended to be private to a group of modules.
* Package visibility
    * This is the the .NET CLR model of “internal” visibility, i.e., things that get compiled together can access each other's internal state based merely on the fact that they are compiled together. In Move, “compiled together” really means published together, so a bundle would require a change in publishing (and the module publish transaction) that would have to accept a list of modules rather than a single one. That would give the verifier a chance to control access across modules. That is, cross module calls towards internal visibility would be allowed if the modules are in the same publish unit (bundle).
    * However, without some extra information to identify the modules in a bundle, this approach implies that visibility/accessibility cannot be verified after publishing (e.g., while loading). The VM would have to assume that every internal access is good because it has no way to verify it after the publishing time.
    * Versioning or upgrades could create problems with this model too, if a version would leave behind some modules that would keep visibility permission but were not intended to. That is something to analyze in more detail and it may or may not be a problem. Essentially the problem is whether verification at publishing time is enough to ensure correctness. It may be argued that the VM knowing the module bundle before and after could be enough to build the dependency graph before and after, and to report errors if any permission inconsistency is detected.
    * Alternatively, modules could declare the “bundle” to which they belong. The binary format could have an entry for the set of modules published together, and those would define the scope when checking internal visibility/accessibility. The bundle would still be published together but then the bytecode verifier would have knowledge of which modules to take into account when verifying internal access.
    * Friend visibility for selected modules fundamentally provides more fine grained access control than package visibility with bundles of modules. And, since Move does not yet have a concept of multi-module packages, friend visibility is a better option.

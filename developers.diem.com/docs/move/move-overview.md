---
title: "Overview"
id: move-overview
hidden: false
---
Move is a next generation language for secure, sandboxed, and formally verified programming. Its first use case is for
the Diem blockchain, where Move provides the foundation for its implementation. However, Move has been developed with
use cases in mind outside a blockchain context as well.

### Start Here

<CardsWrapper cardsPerRow={2}>
    <OverlayCard
        to="/docs/move/move-start-here/move-introduction"
        icon="img/introduction-to-move.svg"
        iconDark="img/introduction-to-move-dark.svg" 
        title="Introduction"
        description="Understand Move’s background, current status and architecture"
    />
    <OverlayCard
        to="/docs/move/move-start-here/move-modules-and-scripts"
        icon="img/modules-and-scripts.svg"
        iconDark="img/modules-and-scripts-dark.svg" 
        title="Modules and Scripts"
        description="Understand Move’s two different types of programs: Modules and Scripts"
    />
    <OverlayCard
        to="/docs/move/move-start-here/move-creating-coins"
        icon="img/diem-coin-sourcing.svg"
        iconDark="img/diem-coin-sourcing-dark.svg" 
        title="First Tutorial: Creating Coins"
        description="Play with Move directly as you create coins with the language"
    />
</CardsWrapper>

### Primitive Types

<CardsWrapper cardsPerRow={2}>
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-integers"
        icon="img/integers-bool.svg"
        iconDark="img/integers-bool-dark.svg" 
        title="Integers"
        description="Move supports three unsigned integer types: u8, u64, and u128"
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-bool"
        icon="img/integers-bool.svg"
        iconDark="img/integers-bool-dark.svg" 
        title="Bool"
        description="Bool is Move's primitive type for boolean true and false values."
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-address"
        icon="img/address.svg"
        iconDark="img/address-dark.svg" 
        title="Address"
        description="Address is a built-in type in Move that is used to represent locations
        in global storage"
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-vector"
        icon="img/vector.svg"
        iconDark="img/vector-dark.svg" 
        title="Vector"
        description="Vector&lt;T&gt; is the only primitive collection type provided by Move"
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-signer"
        icon="img/signer.svg"
        iconDark="img/signer-dark.svg" 
        title="Signer"
        description="Signer is a built-in Move resource type. A signer is a capability that
        allows the holder to act on behalf of a particular address"
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-references"
        icon="img/move-references.svg"
        iconDark="img/move-references-dark.svg" 
        title="References"
        description="Move has two types of references: immutable &amp; and mutable."
    />
    <OverlayCard
        to="/docs/move/move-primitive-types/move-primitives-tuples-unit"
        icon="img/tuples.svg"
        iconDark="img/tuples-dark.svg" 
        title="Tuples and Unit"
        description="In order to support multiple return values, Move has tuple-like
        expressions. We can consider unit() to be an empty tuple"
    />
</CardsWrapper>

### Basic Concepts

<CardsWrapper cardsPerRow={2}>
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-variables"
        icon="img/local-variables-and-scopes.svg"
        iconDark="img/local-variables-and-scopes-dark.svg"
        title="Local Variables and Scopes" 
        description="Local variables in Move are lexically (statically) scoped"
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-abort-assert"
        icon="img/abort-and-return.svg"
        iconDark="img/abort-and-return-dark.svg" 
        title="Abort &amp; Assert"
        description="return and abort are two control flow constructs that end execution, one for the current function and one for the entire transaction"
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-conditionals"
        icon="img/conditionals.svg"
        iconDark="img/conditionals-dark.svg" 
        title="Conditionals" 
        description="An if expression specifies that some code should only be evaluated if a certain condition is true"
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-loops"
        icon="img/loops.svg"
        iconDark="img/loops-dark.svg" 
        title="While and Loop"
        description="Move offers two constructs for looping: while and loop"
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-functions"
        icon="img/functions.svg"
        iconDark="img/functions-dark.svg" 
        title="Functions" 
        description="Function syntax in Move is shared between module functions and script functions"
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-structs-and-resources"
        icon="img/structs-and-resources.svg"
        iconDark="img/structs-and-resources-dark.svg"
        title="Structs and Resources" 
        description="A struct is a user-defined data structure containing typed fields. A resource is a kind of struct that cannot be copied and cannot be dropped"
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-constants"
        icon="img/constants.svg"
        iconDark="img/constants-dark.svg" 
        title="Constants" 
        description="Constants are a way of giving a name to shared, static values inside of a module or script"
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-generics"
        icon="img/generics.svg"
        iconDark="img/generics-dark.svg" 
        title="Generics" 
        description="Generics can be used to define functions and structs over different input data types"
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-equality"
        icon="img/equality.svg"
        iconDark="img/equality-dark.svg" 
        title="Equality"
        description="Move supports two equality operations == and !="
    />
    <OverlayCard
        to="/docs/move/move-basic-concepts/move-basics-uses-aliases"
        icon="img/uses-and-aliases.svg"
        iconDark="img/uses-and-aliases-dark.svg" 
        title="Uses &amp; Aliases"
        description="The use syntax can be used to create aliases to members in othermodules"
    />
</CardsWrapper>

### Global Storage

<CardsWrapper cardsPerRow={2}>
    <OverlayCard
        to="/docs/move/move-global-storage/move-global-storage-structure"
        icon="img/intro-to-global-storage.svg"
        iconDark="img/intro-to-global-storage-dark.svg"
        title="Global Storage Structure"
        description="The purpose of Move programs is to read from and write to persistent global storage"
    />
    <OverlayCard
        to="/docs/move/move-global-storage/move-global-storage-operators"
        icon="img/intro-to-global-storage.svg"
        iconDark="img/intro-to-global-storage-dark.svg"
        title="Global Storage Operators"
        description="Move programs can create, delete, and update resources in global storage using five instructions"
    />
</CardsWrapper>

### Reference

<CardsWrapper cardsPerRow={2}>
    <OverlayCard
        to="/docs/move/move-reference/move-standard-library"
        icon="img/standard-library.svg"
        iconDark="img/standard-library-dark.svg"
        title="Standard Library"
        description="The Move standard library exposes interfaces that implement
        functionality on vectors, option types, error codes and fixed-point
        numbers"
    />
    <OverlayCard
        to="/docs/move/move-reference/move-coding-conventions"
        icon="img/coding-conventions.svg"
        iconDark="img/coding-conventions-dark.svg"
        title="Coding Conventions"
        description="There are basic coding conventions when writing Move code"
    />
</CardsWrapper>
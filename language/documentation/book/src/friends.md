# Friends

The `friend` syntax is used to declare modules that are trusted by the current module.
A trusted module is allowed to call any function defined in the current module that have the `public(friend)` visibility.
For details on function visibilities, please refer to the *Visibility* section in [Functions](./functions.md).

## Friend declaration

A module can declare other modules as friends via friend declaration statements, in the format of

- `friend <address::name>` — friend declaration using fully qualified module name like the example below, or

  ```move
  address 0x42 {
  module A {
      friend 0x42::B;
  }
  }
  ```

- `friend <module-name-alias>` — friend declaration using a module name alias, where the module alias is introduced via the `use` statement.

  ```move
  address 0x42 {
  module A {
      use 0x42::B;
      friend B;
  }
  }
  ```

A module may have multiple friend declarations, and the union of all the friend modules forms the friend list.
In the example below, both `0x42::B` and `0x42::C` are considered as friends of `0x42::A`.

```move
address 0x42 {
module A {
    friend 0x42::B;
    friend 0x42::C;
}
}
```

Unlike `use` statements, `friend` can only be declared in the module scope and not in the expression block scope.
`friend` declarations may be located anywhere a top-level construct (e.g., `use`, `function`, `struct`, etc.) is allowed.
However, for readability, it is advised to place friend declarations near the beginning of the module definition.

Note that the concept of friendship does not apply to Move scripts:
- A Move script cannot declare `friend` modules as doing so is considered meaningless: there is no mechanism to call the function defined in a script.
- A Move module cannot declare `friend` scripts as well because scripts are ephemeral code snippets that are never published to global storage.

### Friend declaration rules
Friend declarations are subject to the following rules:

- A module cannot declare itself as a friend.

  ```move=
  address 0x42 {
  module M { friend Self; // ERROR! }
  //                ^^^^ Cannot declare the module itself as a friend
  }

  address 0x43 {
  module M { friend 0x43::M; // ERROR! }
  //                ^^^^^^^ Cannot declare the module itself as a friend
  }
  ```

- Friend modules must be known by the compiler

  ```move=
  address 0x42 {
  module M { friend 0x42::Nonexistent; // ERROR! }
  //                ^^^^^^^^^^^^^^^^^ Unbound module '0x42::Nonexistent'
  }
  ```

- Friend modules must be within the same account address. (Note: this is not a technical requirement but rather a policy decision which *may* be relaxed later.)

  ```move=
  address 0x42 {
  module M {}
  }

  address 0x43 {
  module N { friend 0x42::M; // ERROR! }
  //                ^^^^^^^ Cannot declare modules out of the current address as a friend
  }
  ```

- Friends relationships cannot create cyclic module dependencies.

  Cycles are not allowed in the friend relationships, e.g., the relation `0x2::A` friends `0x2::B` friends `0x2::C` friends `0x2::A` is not allowed.
More generally, declaring a friend module adds a dependency upon the current module to the friend module (because the purpose is for the friend to call functions in the current module).
If that friend module is already used, either directly or transitively, a cycle of dependencies would be created.
  ```move=
  address 0x2 {
  module A {
      use 0x2::C;
      friend 0x2::B;

      public fun a() {
          C::c()
      }
  }

  module B {
      friend 0x2::C; // ERROR!
  //         ^^^^^^ This friend relationship creates a dependency cycle: '0x2::B' is a friend of '0x2::A' uses '0x2::C' is a friend of '0x2::B'
  }

  module C {
      public fun c() {}
  }
  }
  ```

- The friend list for a module cannot contain duplicates.

  ```move=
  address 0x42 {
  module A {}

  module M {
      use 0x42::A as AliasedA;
      friend 0x42::A;
      friend AliasedA; // ERROR!
  //         ^^^^^^^^ Duplicate friend declaration '0x42::A'. Friend declarations in a module must be unique
  }
  }
  ```

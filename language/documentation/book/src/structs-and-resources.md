# Structs and Resources

A *struct* is a user-defined data structure containing typed fields. Structs can store any non-reference type, including other structs.

We often refer to struct values as *resources* if they cannot be copied and cannot be dropped. In this case, resource values must have ownership transferred by the end of the function. This property makes resources particularly well served for defining global storage schemas or for representing important values (such as a token).

By default, structs are linear and ephemeral. By this we mean that they: cannot be copied, cannot be dropped, and cannot be stored in global storage. This means that all values have to have ownership transferred (linear) and the values must be dealt with by the end of the program's execution (ephemeral). We can relax this behavior by giving the struct [abilities](./abilities.md) which allow values to be copied or dropped and also to be stored in global storage or to define gobal storage schemas.

## Defining Structs

Structs must be defined inside a module:

```move
address 0x2 {
module M {
    struct Foo { x: u64, y: bool }
    struct Bar {}
    struct Baz { foo: Foo, }
    //                   ^ note: it is fine to have a trailing comma
}
}
```

Structs cannot be recursive, so the following definition is invalid:

```move=
struct Foo { x: Foo }
//              ^ error! Foo cannot contain Foo
```

As mentioned above: by default, a struct declaration is linear and ephemeral. So to allow the value to be used with certain operations (that copy it, drop it, store it in global storage, or use it as a storage schema), structs can be granted [abilities](./abilities.md) by annotating them with `has <ability>`:

```move=
address 0x2 {
module M {
    struct Foo has copy, drop { x: u64, y: bool }
}
}
```

For more details, see the [annotating structs](./abilities.md#annotating-structs) section.

### Naming

Structs must start with a capital letter `A` to `Z`. After the first letter, constant names can contain underscores `_`, letters `a` to `z`, letters `A` to `Z`, or digits `0` to `9`.

```move
struct Foo {}
struct BAR {}
struct B_a_z_4_2 {}
```

This naming restriction of starting with `A` to `Z` is in place to give room for future language features. It may or may not be removed later.

## Using Structs

### Creating Structs

Values of a struct type can be created (or "packed") by indicating the struct name, followed by value for each field:

```move=
address 0x2 {
module M {
    struct Foo has drop { x: u64, y: bool }
    struct Baz has drop { foo: Foo }

    fun example() {
        let foo = Foo { x: 0, y: false };
        let baz = Baz { foo: foo };
    }
}
}
```

If you initialize a struct field with a local variable whose name is the same as the field, you can use the following shorthand:

```move
let baz = Baz { foo: foo };
// is equivalent to
let baz = Baz { foo };
```

This is called sometimes called "field name punning".

### Destroying Structs via Pattern Matching

Struct values can be destroyed by binding or assigning them patterns.

```move=
address 0x2 {
module M {
    struct Foo { x: u64, y: bool }
    struct Bar { foo: Foo }
    struct Baz {}

    fun example_destroy_foo() {
        let foo = Foo { x: 3, y: false };
        let Foo { x, y: foo_y } = foo;
        //        ^ shorthand for `x: x`

        // two new bindings
        //   x: u64 = 3
        //   foo_y: bool = false
    }

    fun example_destroy_foo_wildcard() {
        let foo = Foo { x: 3, y: false };
        let Foo { x, y: _ } = foo;
        // only one new binding since y was bound to a wildcard
        //   x: u64 = 3
    }

    fun example_destroy_foo_assignment() {
        let x: u64;
        let y: bool;
        Foo { x, y } = Foo { x: 3, y: false };
        // mutating existing variables x & y
        //   x = 3, y = false
    }

    fun example_foo_ref() {
        let foo = Foo { x: 3, y: false };
        let Foo { x, y } = &foo;
        // two new bindings
        //   x: &u64
        //   y: &bool
    }

    fun example_foo_ref_mut() {
        let foo = Foo { x: 3, y: false };
        let Foo { x, y } = &mut foo;
        // two new bindings
        //   x: &mut u64
        //   y: &mut bool
    }

    fun example_destroy_bar() {
        let bar = Bar { foo: Foo { x: 3, y: false } };
        let Bar { foo: Foo { x, y } } = bar;
        //             ^ nested pattern
        // two new bindings
        //   x: u64 = 3
        //   foo_y: bool = false
    }

    fun example_destroy_baz() {
        let baz = Baz {};
        let Baz {} = baz;
    }
}
}
```

### Borrowing Structs and Fields

The `&` and `&mut` operator can be used to create references to structs or fields. These examples include some optional type annotations (e.g., `: &Foo`) to demonstrate the type of operations.

```move=
let foo = Foo { x: 3, y: true };
let foo_ref: &Foo = &foo;
let y: bool = foo_ref.y;          // reading a field via a reference to the struct
let x_ref: &u64 = &foo.x;

let x_ref_mut: &mut u64 = &mut foo.x;
*x_ref_mut = 42;            // modifying a field via a mutable reference
```

It is possible to borrow inner fields of nested structs.

```move=
let foo = Foo { x: 3, y: true };
let bar = Bar { foo };

let x_ref = &bar.foo.x;
```

You can also borrow a field via a reference to a struct.

```move=
let foo = Foo { x: 3, y: true };
let foo_ref = &foo;
let x_ref = &foo_ref.x;
// this has the same effect as let x_ref = &foo.x
```

### Reading and Writing Fields

If you need to read and copy a field's value, you can then dereference the borrowed field

```move=
let foo = Foo { x: 3, y: true };
let bar = Bar { foo: copy foo };
let x: u64 = *&foo.x;
let y: bool = *&foo.y;
let foo2: Foo = *&bar.foo;
```

If the field is implicitly copyable, the dot operator can be used to read fields of a struct without any borrowing. (Only scalar values with the `copy` ability are implicitly copyable.)

```move=
let foo = Foo { x: 3, y: true };
let x = foo.x;  // x == 3
let y = foo.y;  // y == true
```

Dot operators can be chained to access nested fields.

```move=
let baz = Baz { foo: Foo { x: 3, y: true } };
let x = baz.foo.x; // x = 3;
```

However, this is not permitted for fields that contain non-primitive types, such a vector or another struct

```move=
let foo = Foo { x: 3, y: true };
let bar = Bar { foo };
let foo2: Foo = *&bar.foo;
let foo3: Foo = bar.foo; // error! add an explicit copy with *&
```

The reason behind this design decision is that copying a vector or another struct might be an expensive operation. It is important for a programmer to be aware of this copy and make others aware with the explicit syntax `*&`

In addition reading from fields, the dot syntax can be used to modify fields, regardless of the field being a primitive type or some other struct

```move=
let foo = Foo { x: 3, y: true };
foo.x = 42;     // foo = Foo { x: 42, y: true }
foo.y = !foo.y; // foo = Foo { x: 42, y: false }
let bar = Bar { foo };            // bar = Bar { foo: Foo { x: 42, y: false } }
bar.foo.x = 52;                   // bar = Bar { foo: Foo { x: 52, y: false } }
bar.foo = Foo { x: 62, y: true }; // bar = Bar { foo: Foo { x: 62, y: true } }
```

The dot syntax also works via a reference to a struct

```move=
let foo = Foo { x: 3, y: true };
let foo_ref = &mut foo;
foo_ref.x = foo_ref.x + 1;
```

## Privileged Struct Operations

Most struct operations on a struct type `T` can only be performed inside the module that declares `T`:

- Struct types can only be created ("packed"), destroyed ("unpacked") inside the module that defines the struct.
- The fields of a struct are only accessible inside the module that defines the struct.

Following these rules, if you want to modify your struct outside the module, you will need to provide publis APIs for them. The end of the chapter contains some examples of this.

However, struct *types* are always visible to another module or script:

```move=
// M.move
address 0x2 {
module M {
    struct Foo has drop { x: u64 }

    public fun new_foo(): Foo {
        Foo { x: 42 }
    }
}
}
```

```move=
// N.move
address 0x2 {
module N {
    use 0x2::M;

    struct Wrapper has drop {
        foo: M::Foo
    }

    fun f1(foo: M::Foo) {
        let x = foo.x;
        //      ^ error! cannot access fields of `foo` here
    }

    fun f2() {
        let foo_wrapper = Wrapper { foo: M::new_foo() };
    }
}
}
```

Note that structs do not have visibility modifiers (e.g., `public` or `private`).

## Ownership

As mentioned above in [Defining Structs](#defining-structs), structs are by default linear and ephemeral. This means they cannot be copied or dropped. This property can be very useful when modeling real world resources like money, as you do not want money to be duplicated or get lost in circulation.

```move=
address 0x2 {
module M {
    struct Foo { x: u64 }

    public fun copying_resource() {
        let foo = Foo { x: 100 };
        let foo_copy = copy foo; // error! 'copy'-ing requires the 'copy' ability
        let foo_ref = &foo;
        let another_copy = *foo_ref // error! dereference requires the 'copy' ability
    }

    public fun destroying_resource1() {
        let foo = Foo { x: 100 };

        // error! when the function returns, foo still contains a value.
        // This destruction requires the 'drop' ability
    }

    public fun destroying_resource2(f: &mut Foo) {
        *f = Foo { x: 100 } // error!
                            // destroying the old value via a write requires the 'drop' ability
    }
}
}
```

To fix the second example (`fun dropping_resource`), you would need to manually "unpack" the resource:

```move=
address 0x2 {
module M {
    struct Foo { x: u64 }

    public fun destroying_resource1_fixed() {
        let foo = Foo { x: 100 };
        let Foo { x: _ } = foo;
    }
}
}
```

Recall that you are only able to deconstruct a resource within the module in which it is defined.
This can be leveraged to enforce certain invariants in a system, for example, conservation of money.

If on the other hand, your struct does not represent something valuable, you can add the abilities
`copy` and `drop` to get a struct value that might feel more familiar from other programming
languages:

```move=
address 0x2 {
module M {
    struct Foo has copy, drop { x: u64 }

    public fun run() {
        let foo = Foo { x: 100 };
        let foo_copy = copy foo;
        // ^ this code copies foo, whereas `let x = foo` or
        // `let x = move foo` both move foo

        let x = foo.x;            // x = 100
        let x_copy = foo_copy.x;  // x = 100

        // both foo and foo_copy are implicitly discarded when the function returns
    }
}
}
```

## Storing Resources in Global Storage

Only structs with the `key` ability can be saved directly in [persistent global storage](./global-storage-operators.md). All values stored within those `key` structs must have the `store` abilities. See the [ability](./abilities] and [global storage](./global-storage-operators.md) chapters for more detail.

## Examples

Here are two short examples of how you might use structs to represent valuable data (in the case of `Coin`) or more classical data (in the case of `Point` and `Circle`)

### Example 1: Coin

<!-- TODO link to access control for mint -->
```move=
address 0x2 {
module M {
    // We do not want the Coin to be copied because that would be duplicating this "money",
    // so we do not give the struct the 'copy' ability.
    // Similarly, we do not want programmers to destroy coins, so we do not give the struct the
    // 'drop' ability.
    // However, we *want* users of the modules to be able to store this coin in persistent global
    // storage, so we grant the struct the 'store' ability. This struct will only be inside of
    // other resources inside of global storage, so we do not give the struct the 'key' ability.
    struct Coin has store {
        value: u64,
    }

    public fun mint(value: u64): Coin {
        // You would want to gate this function with some form of access control to prevent
        // anyone using this module from minting an infinite amount of coins
        Coin { value }
    }

    public fun withdraw(coin: &mut Coin, amount: u64): Coin {
        assert(coin.balance >= amount, 1000);
        coin.value = coin.value - amount;
        Coin { value: amount }
    }

    public fun deposit(coin: &mut Coin, other: Coin) {
        let Coin { value } = other;
        coin.value = coin.value + value;
    }

    public fun split(coin: Coin, amount: u64): (Coin, Coin) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }

    public fun merge(coin1: Coin, coin2: Coin): Coin {
        deposit(&mut coin1, coin2);
        coin1
    }

    public fun destroy_zero(coin: Coin) {
        let Coin { value } = coin;
        assert(value == 0, 1001);
    }
}
}
```

### Example 2: Geometry

```move=
address 0x2 {
module Point {
    struct Point has copy, drop, store {
        x: u64,
        y: u64,
    }

    public fun new(x: u64, y: u64): Point {
        Point {
            x, y
        }
    }

    public fun x(p: &Point): u64 {
        p.x
    }

    public fun y(p: &Point): u64 {
        p.y
    }

    fun abs_sub(a: u64, b: u64): u64 {
        if (a < b) {
            b - a
        }
        else {
            a - b
        }
    }

    public fun dist_squared(p1: &Point, p2: &Point): u64 {
        let dx = abs_sub(p1.x, p2.x);
        let dy = abs_sub(p1.y, p2.y);
        dx*dx + dy*dy
    }
}
}
```

```move=
address 0x2 {
module Circle {
    use 0x2::Point::{Self, Point};

    struct Circle has copy, drop, store {
        center: Point,
        radius: u64,
    }

    public fun new(center: Point, radius: u64): Circle {
        Circle { center, radius }
    }

    public fun overlaps(c1: &Circle, c2: &Circle): bool {
        let d = Point::dist_squared(&c1.center, &c2.center);
        let r1 = c1.radius;
        let r2 = c2.radius;
        d*d <= r1*r1 + 2*r1*r2 + r2*r2
    }
}
}
```

---
id: transaction-builder-generator
title: Transaction Builder Generator
custom_edit_url: https://github.com/diem/diem/edit/main/language/transaction-builder-generator/README.md
---

# Transaction Builder Generator

A *transaction builder* is a helper function that converts its arguments into the payload of a Diem transaction calling a particular Move script.

In Rust, the signature of such a function typically looks like this:
```rust
pub fn encode_peer_to_peer_with_metadata_script(
    token: TypeTag,
    payee: AccountAddress,
    amount: u64,
    metadata: Vec<u8>,
    metadata_signature: Vec<u8>,
) -> Script;
```

This crate provide a binary tool `generate-transaction-builders` to generate and install transaction builders in several programming languages.

The tool will also generate and install type definitions for Diem types such as `TypeTag`, `AccountAddress`, and `Script`.

In practice, hashing and signing Diem transactions additionally requires a runtime library for Binary Canonical Serialization ("BCS").
Such a library will be installed together with the Diem types.


## Supported Languages

The following languages are currently supported:

* Python 3

* C++ 17

* Java 8

* Go >= 1.13

* Rust (NOTE: Code generation of dependency-free Rust is experimental. Consider using the libraries of the Diem repository instead.)

* TypeScript / JavaScript


## Quick Start

From the root of the Diem repository, run `cargo build -p transaction-builder-generator`.

You may browse command line options with `target/debug/generate-transaction-builders --help`.

NOTE: until the Diem version flag is set to greater than `2` the path
used for generating transaction builders should be
`language/diem-framework/legacy/transaction_scripts/abi`. You can query
this version number by submitting a `get_metadata` request to the JSON-RPC
endpoint.

### Python

To install Python3 modules `serde`, `bcs`, `diem_types`, and `diem_framework` into a target directory `$DEST`, run:
```bash
target/debug/generate-transaction-builders \
    --language python3 \
    --module-name diem_framework \
    --with-diem-types "testsuite/generate-format/tests/staged/diem.yaml" \
    --target-source-dir "$DEST" \
    --with-custom-diem-code language/transaction-builder/generator/examples/python3/custom_diem_code/*.py -- \
    "language/diem-framework/releases/legacy" \
    "language/diem-framework/releases/artifacts/current"
```
Next, you may copy and execute the [Python demo file](examples/python3/stdlib_demo.py) with:
```bash
cp language/transaction-builder/generator/examples/python3/stdlib_demo.py "$DEST"
PYTHONPATH="$PYTHONPATH:$DEST" python3 "$DEST/stdlib_demo.py"
```

### C++

To install C++ files `serde.hpp`, `bcs.hpp`, `diem_types.hpp`, `diem_framework.hpp`, `diem_framework.cpp` into a target directory `$DEST`, run:
```bash
target/debug/generate-transaction-builders \
    --language cpp \
    --module-name diem_framework \
    --with-diem-types "testsuite/generate-format/tests/staged/diem.yaml" \
    --target-source-dir "$DEST" \
    "language/diem-framework/releases/legacy" \
    "language/diem-framework/releases/artifacts/current"
```
Next, you may copy and execute the [C++ demo file](examples/cpp/stdlib_demo.cpp) with:
```bash
cp language/transaction-builder/generator/examples/cpp/stdlib_demo.cpp "$DEST"
clang++ --std=c++17 -I "$DEST" "$DEST/diem_framework.cpp" "$DEST/stdlib_demo.cpp" -o "$DEST/stdlib_demo"
"$DEST/stdlib_demo"
```

### Java

To install Java source packages `com.novi.serde`, `com.novi.bcs`, `com.diem.types`, and `com.diem.stdlib` into a target directory `$DEST`, run:
```bash
target/debug/generate-transaction-builders \
    --language java \
    --module-name com.diem.stdlib \
    --with-diem-types "testsuite/generate-format/tests/staged/diem.yaml" \
    --target-source-dir "$DEST" \
    --with-custom-diem-code language/transaction-builder/generator/examples/java/custom_diem_code/*.java -- \
    "language/diem-framework/releases/legacy" \
    "language/diem-framework/releases/artifacts/current"
```
Next, you may copy and execute the [Java demo file](examples/java/StdlibDemo.java) with:
```bash
cp language/transaction-builder/generator/examples/java/StdlibDemo.java "$DEST"
(find "$DEST" -name "*.java" | xargs javac -cp "$DEST")
java -enableassertions -cp "$DEST" StdlibDemo
```

### Go

To generate the Go "packages" `testing/diemtypes`, and `testing/diemstdlib` into a target directory `$DEST`, run:

```bash
target/debug/generate-transaction-builders \
    --language go \
    --module-name diemstdlib \
    --diem-package-name testing \
    --with-diem-types "testsuite/generate-format/tests/staged/diem.yaml" \
    --target-source-dir "$DEST" \
    "language/diem-framework/releases/legacy" \
    "language/diem-framework/releases/artifacts/current"
```
Next, you may copy and execute the [Go demo file](examples/golang/stdlib_demo.go) as follows:
(Note that `$DEST` must be an absolute path)
```bash
cp language/transaction-builder/generator/examples/golang/stdlib_demo.go "$DEST"
(cd "$DEST" && go mod init testing && go mod edit -replace testing="$DEST" && go run stdlib_demo.go)
```

### Rust (experimental)

To install dependency-free Rust crates `diem-types` and `diem-framework` into a target directory `$DEST`, run:
```bash
target/debug/generate-transaction-builders \
    --language rust \
    --module-name diem-framework \
    --with-diem-types "testsuite/generate-format/tests/staged/diem.yaml" \
    --target-source-dir "$DEST" \
    "language/diem-framework/releases/legacy" \
    "language/diem-framework/releases/artifacts/current"
```
Next, you may copy and execute the [Rust demo file](examples/rust/stdlib_demo.rs). (See [unit test](tests/generation.rs) for details.)

### TypeScript/JavaScript

To generate the TypeScript "module" `diemStdlib` and its submodules into a target directory `$DEST`, run:

```bash
target/debug/generate-transaction-builders \
    --language typescript \
    --module-name diemStdlib \
    --with-diem-types "testsuite/generate-format/tests/staged/diem.yaml" \
    --target-source-dir "$DEST" \
    "language/diem-framework/releases/legacy" \
    "language/diem-framework/releases/artifacts/current"
```

### C#

To install C# source `Serde`, `Bcs`, `Diem.Types`, and `Diem.Stdlib` into a target directory `$DEST`, run:
```bash
target/debug/generate-transaction-builders \
    --language csharp \
    --module-name Diem.Stdlib \
    --with-diem-types "testsuite/generate-format/tests/staged/diem.yaml" \
    --target-source-dir "$DEST" \
    --with-custom-diem-code language/transaction-builder/generator/examples/csharp/custom_diem_code/*.cs -- \
    "language/diem-framework/releases/legacy" \
    "language/diem-framework/releases/artifacts/current"
```
Next, you may copy and execute the [C# demo file](examples/csharp/StdlibDemo.cs) with:
```bash
mkdir "$DEST"/Demo
cp language/transaction-builder/generator/examples/csharp/StdlibDemo.cs "$DEST/Demo"
cd "$DEST/Diem/Stdlib"
dotnet new classlib -n Diem.Stdlib -o .
dotnet add Diem.Stdlib.csproj reference ../Types/Diem.Types.csproj
rm Class1.cs
cd "../../Demo"
dotnet new sln
dotnet new console
rm Program.cs
dotnet add Demo.csproj reference ../Diem/Stdlib/Diem.Stdlib.csproj
dotnet add Demo.csproj reference ../Diem/Types/Diem.Types.csproj
dotnet add Demo.csproj reference ../Serde/Serde.csproj
dotnet add Demo.csproj reference ../Bcs/Bcs.csproj
dotnet run --project Demo.csproj
```

## Adding Support for a New Language

Supporting transaction builders in an additional programming language boils down to providing the following items:

1. Code generation for Diem types (Rust library and tool),

2. BCS runtime (library in target language),

3. Code generation for transaction builders (Rust tool).


Items (1) and (2) are provided by the Rust library `serde-generate` which is developed in a separate [github repository]({ git = "https://github.com/diem/serde-reflection-diem.git"}).

Item (3) --- this tool --- is currently developed in the Diem repository.

Items (2) and (3) are mostly independent. Both crucially depend on (1) to be sufficiently stable, therefore our suggestion for adding a new language is first to open a new github issue in `serde-generate` and contact the Diem maintainers.


The new issue created on `serde-generate` should include:

* requirements for the production environment (e.g. Java >= 8);

* suggestions for a testing environment in CircleCI (if not easily available in Debian 10.4 "buster");

* optionally, suggested definitions for a sample of types in the target language (structs, enums, arrays, tuples, vectors, maps, (un)signed integers of various sizes, etc).

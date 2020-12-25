# ssf

[![GitHub Action](https://img.shields.io/github/workflow/status/raviqqe/ssf/test?style=flat-square)](https://github.com/raviqqe/ssf/actions?query=workflow%3Atest)
[![Codecov](https://img.shields.io/codecov/c/github/raviqqe/ssf.svg?style=flat-square)](https://codecov.io/gh/raviqqe/ssf)
[![License](https://img.shields.io/github/license/raviqqe/ssf.svg?style=flat-square)](LICENSE)

`ssf` is a structurally-typed strict functional core language supposed to be used as a target language for high-level strict functional programming languages.

This repository consists of two crates of `ssf` and `ssf-llvm`. The former is to construct intermediate representation (IR) of `ssf` going through type check and other validation and the latter is to compile it into LLVM IR bitcode.

## Install

In your `Cargo.toml`,

```
ssf = { git = "https://github.com/raviqqe/ssf", branch = "master" }
ssf-llvm = { git = "https://github.com/raviqqe/ssf", branch = "master" }
```

## Features

- Inference of closure environment types
- Partial application
- Bitcast
- Lazy evaluation

### Ones not supported...

- Type inference
  - The IR needs to be fully-typed already.
- Generics
- Garbage collection
  - Bring your own GC.

## Type system

- Functions
- Algebraic data types
  - Constructors are boxed or unboxed explicitly.
- Primitives
  - 8-bit integer
  - 32-bit integer
  - 64-bit integer
  - 32-bit floating point number
  - 64-bit floating point number

### Binary representation of ADTs

- Tags are pointer-sized integers.
- Constructor payloads boxed or unboxed contain their elements.

#### Single constructor with no payload

- Empty data

#### Single constructor with payload

| (payload size) |
| -------------- |
| payload        |

#### Multiple constructors with no payload

| (pointer size) |
| -------------- |
| tag            |

#### Multiple constructors with payload

| (pointer size) | (max payload size) |
| -------------- | ------------------ |
| tag            | payload            |

## Examples

```rust
let algebraic_type = ssf::types::Algebraic::new(vec![ssf::types::Constructor::boxed(vec![
    ssf::types::Primitive::Float64.into(),
])]);

let bitcode = ssf_llvm::compile(
    &ssf::ir::Module::new(
        vec![],
        vec![ssf::ir::FunctionDefinition::new(
            "f",
            vec![ssf::ir::Argument::new("x", ssf::types::Primitive::Float64)],
            ssf::ir::ConstructorApplication::new(
                ssf::ir::Constructor::boxed(algebraic_type.clone(), 0),
                vec![ssf::ir::Variable("x").into()],
            ),
            algebraic_type,
        )
        .into()],
    )
    .unwrap(),
    &CompileConfiguration::new(None, None),
)?;
```

## License

[MIT](LICENSE)

# justerror
[<img alt="github" src="https://img.shields.io/badge/github-shakacode/justerror-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/shakacode/justerror)
[<img alt="crates.io" src="https://img.shields.io/crates/v/justerror.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/justerror)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-justerror-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/justerror)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/shakacode/justerror/CI/main?style=for-the-badge" height="20">](https://github.com/shakacode/justerror/actions?query=branch%3Amain)
<!-- cargo-sync-readme start -->

This macro piggybacks on [`thiserror`](https://github.com/dtolnay/thiserror) crate and is supposed to reduce the amount of handwriting when you want errors in your app to be described via explicit types (rather than [`anyhow`](https://github.com/dtolnay/anyhow)).

## Installation

Add to `Cargo.toml`:

```toml
justerror = "0.1"
```

Add to `main.rs`:

```rust
#[macro_use]
extern crate justerror;
```

## Usage
This macro takes a subject struct or enum and applies `thiserror` attributes with predefined `#[error]` messages.

Generally, you can attach `#[Error]` macro to an error type and be done with it.

```rust
#[Error]
enum EnumError {
    Foo,
    Bar {
        a: &'static str,
        b: usize
    },
}

eprintln!("{}", EnumError::Bar { a: "Hey!", b: 42 });

// EnumError::Bar
// === DEBUG DATA:
// a: Hey!
// b: 42
```

Macro accepts two optional arguments:
- `desc`: string
- `fmt`: `display` | `debug` | `"<custom format>"`

Both can be applied at the root level.

```rust
#[Error(desc = "My emum error description", fmt = debug)]
enum EnumError {
    Foo(usize),
}
```

And at the variant level.

```rust
#[Error(desc = "My emum error description", fmt = debug)]
enum EnumError {
    #[error(desc = "Foo error description", fmt = display)]
    Foo(usize),
}
```

`fmt` can also be applied to individual fields.

```rust
#[Error(desc = "My emum error description", fmt = debug)]
enum EnumError {
    #[error(desc = "Foo error description", fmt = display)]
    Foo(#[fmt(">5")] usize),
}
```

See [tests](tests/tests.rs) for more examples.

<!-- cargo-sync-readme end -->

## License
MIT.
# justerror
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

## License
MIT.
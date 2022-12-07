//! This macro piggybacks on [`thiserror`](https://github.com/dtolnay/thiserror) crate and is supposed to reduce the amount of handwriting when you want errors in your app to be described via explicit types (rather than [`anyhow`](https://github.com/dtolnay/anyhow)).
//!
//! ## Installation
//!
//! Add to `Cargo.toml`:
//!
//! ```toml
//! justerror = "0.1"
//! ```
//!
//! Add to `main.rs`:
//!
//! ```ignore
//! #[macro_use]
//! extern crate justerror;
//! ```
//!
//! ## Usage
//! This macro takes a subject struct or enum and applies `thiserror` attributes with predefined `#[error]` messages.
//!
//! Generally, you can attach `#[Error]` macro to an error type and be done with it.
//!
//! ```rust
//! # use justerror::Error;
//! #[Error]
//! enum EnumError {
//!     Foo,
//!     Bar {
//!         a: &'static str,
//!         b: usize
//!     },
//! }
//!
//! eprintln!("{}", EnumError::Bar { a: "Hey!", b: 42 });
//!
//! // EnumError::Bar
//! // === DEBUG DATA:
//! // a: Hey!
//! // b: 42
//! ```
//!
//! Macro accepts two optional arguments:
//! - `desc`: string
//! - `fmt`: `display` | `debug` | `"<custom format>"`
//!
//! Both can be applied at the root level.
//!
//! ```rust
//! # use justerror::Error;
//! #[Error(desc = "My emum error description", fmt = debug)]
//! enum EnumError {
//!     Foo(usize),
//! }
//! ```
//!
//! And at the variant level.
//!
//! ```rust
//! # use justerror::Error;
//! #[Error(desc = "My emum error description", fmt = debug)]
//! enum EnumError {
//!     #[error(desc = "Foo error description", fmt = display)]
//!     Foo(usize),
//! }
//! ```
//!
//! `fmt` can also be applied to individual fields.
//!
//! ```rust
//! # use justerror::Error;
//! #[Error(desc = "My emum error description", fmt = debug)]
//! enum EnumError {
//!     #[error(desc = "Foo error description", fmt = display)]
//!     Foo(#[fmt(">5")] usize),
//! }
//! ```
//!
//! See [tests](tests/tests.rs) for more examples.

extern crate proc_macro;

use std::{
    default::Default,
    fmt::{self, Display},
};

use proc_macro::TokenStream as CompilerTokenStream;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Data, DeriveInput, Error as SyntaxError, Field, Fields, Ident,
    Lit, Token,
};

const ERROR_ATTR: &str = "error";
const FMT_ATTR: &str = "fmt";

mod kw {
    syn::custom_keyword!(desc);
    syn::custom_keyword!(fmt);
    syn::custom_keyword!(debug);
    syn::custom_keyword!(display);
}

#[derive(Default, Debug)]
struct ErrorArgs {
    desc: Option<String>,
    fmt: Option<Fmt>,
}

impl ErrorArgs {
    fn parse_desc(input: ParseStream) -> syn::Result<String> {
        let _: kw::desc = input.parse()?;
        let _: Token![=] = input.parse()?;
        let val: Lit = input.parse()?;

        match val {
            Lit::Str(str) => Ok(str.value()),
            _ => Err(SyntaxError::new(val.span(), "`desc` must be a string")),
        }
    }

    fn parse_fmt(input: ParseStream) -> syn::Result<Fmt> {
        let _: kw::fmt = input.parse()?;
        let _: Token![=] = input.parse()?;
        let val = input.parse::<Fmt>()?;

        Ok(val)
    }
}

impl Parse for ErrorArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut args = Self::default();

        let lookahead = input.lookahead1();

        if lookahead.peek(kw::desc) {
            let desc = Self::parse_desc(input)?;
            args.desc = Some(desc);
        } else if lookahead.peek(kw::fmt) {
            let fmt = Self::parse_fmt(input)?;
            args.fmt = Some(fmt);
        } else {
            return Err(lookahead.error());
        }

        if input.is_empty() {
            return Ok(args);
        } else {
            input.parse::<Token![,]>()?;
        }

        let lookahead = input.lookahead1();

        if lookahead.peek(kw::desc) {
            if args.desc.is_some() {
                return Err(SyntaxError::new(input.span(), "`desc` is already defined"));
            }
            let desc = Self::parse_desc(input)?;
            args.desc = Some(desc);
        } else if lookahead.peek(kw::fmt) {
            if args.fmt.is_some() {
                return Err(SyntaxError::new(input.span(), "`fmt` is already defined"));
            }
            let fmt = Self::parse_fmt(input)?;
            args.fmt = Some(fmt);
        } else {
            return Err(lookahead.error());
        }

        if input.is_empty() {
            Ok(args)
        } else {
            Err(SyntaxError::new(
                input.span(),
                "`error` can't have more than 2 arguments",
            ))
        }
    }
}

#[derive(Clone, Debug)]
enum Fmt {
    Display,
    Debug,
    Custom(String),
}

impl Fmt {
    fn derive(root: &ErrorArgs, variant: &Option<ErrorArgs>, field: &Option<Self>) -> Self {
        match field {
            Some(fmt) => fmt.to_owned(),
            None => match variant {
                Some(ErrorArgs {
                    desc: _,
                    fmt: Some(fmt),
                }) => fmt.to_owned(),
                Some(_) | None => match &root.fmt {
                    Some(fmt) => fmt.to_owned(),
                    None => Fmt::default(),
                },
            },
        }
    }
}

impl Default for Fmt {
    fn default() -> Self {
        Fmt::Display
    }
}

impl Display for Fmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fmt::Display => Ok(()),
            Fmt::Debug => write!(f, ":#?"),
            Fmt::Custom(fmt) => write!(f, ":{}", fmt),
        }
    }
}

impl Parse for Fmt {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let fmt = match input.parse::<kw::debug>() {
            Ok(_) => Fmt::Debug,
            Err(_) => match input.parse::<kw::display>() {
                Ok(_) => Fmt::Display,
                Err(_) => match input.parse::<Lit>()? {
                    Lit::Str(str) => Fmt::Custom(str.value()),
                    lit => {
                        return Err(SyntaxError::new(
                            lit.span(),
                            "`fmt` must be either `debug`, `display` or a custom string",
                        ))
                    }
                },
            },
        };

        Ok(fmt)
    }
}

struct Output(String);

enum FieldIdentStyle {
    Prefixed,
    Unprefixed,
}

impl Output {
    fn new() -> Self {
        Self(String::new())
    }

    fn push_title(&mut self, head: &Ident, tail: Option<&Ident>) {
        let buf = &mut self.0;

        buf.push_str(&head.to_string());

        if let Some(tail) = tail {
            buf.push_str("::");
            buf.push_str(&tail.to_string());
        }
    }

    fn push_desc(&mut self, prefix: Option<&Ident>, desc: &str) {
        let buf = &mut self.0;

        buf.push('\n');

        if let Some(prefix) = prefix {
            buf.push_str(&prefix.to_string());
            buf.push_str(": ");
        }

        buf.push_str(desc);
    }

    fn push_debug_title(&mut self) {
        let buf = &mut self.0;

        buf.push('\n');
        buf.push_str("=== DEBUG DATA:");
    }

    fn push_fields(
        &mut self,
        fields: &mut Fields,
        error_args: &ErrorArgs,
        variant_error_args: &Option<ErrorArgs>,
    ) -> Result<(), TokenStream> {
        let output = self;

        match fields {
            Fields::Named(fields) => {
                output.push_debug_title();

                for field in &mut fields.named {
                    if let Some(field_ident) = field.ident.clone() {
                        output.push_field(
                            field,
                            &field_ident,
                            &FieldIdentStyle::Prefixed,
                            error_args,
                            variant_error_args,
                        )?;
                    }
                }
            }
            Fields::Unnamed(fields) => {
                output.push_debug_title();

                let ident_style = if fields.unnamed.len() > 1 {
                    FieldIdentStyle::Prefixed
                } else {
                    FieldIdentStyle::Unprefixed
                };

                for (idx, field) in fields.unnamed.iter_mut().enumerate() {
                    output.push_field(field, idx, &ident_style, error_args, variant_error_args)?;
                }
            }
            Fields::Unit => (),
        }

        Ok(())
    }

    fn push_field(
        &mut self,
        field: &mut Field,
        ident: impl Display,
        ident_style: &FieldIdentStyle,
        error_args: &ErrorArgs,
        variant_error_args: &Option<ErrorArgs>,
    ) -> Result<(), TokenStream> {
        let mut field_fmt_attr = None;

        for (idx, attr) in field.attrs.iter().enumerate() {
            if attr.path.is_ident(FMT_ATTR) {
                field_fmt_attr = match attr.parse_args::<Fmt>() {
                    Ok(fmt) => Some((idx, fmt)),
                    Err(err) => return Err(err.into_compile_error()),
                };
            }
        }

        let (field_fmt_attr_idx, field_fmt) = match field_fmt_attr {
            Some((idx, fmt)) => (Some(idx), Some(fmt)),
            None => (None, None),
        };

        if let Some(idx) = field_fmt_attr_idx {
            field.attrs.remove(idx);
        }

        let fmt = Fmt::derive(error_args, variant_error_args, &field_fmt);

        let buf = &mut self.0;

        let ident = ident.to_string();
        let fmt = fmt.to_string();

        buf.push('\n');

        if let FieldIdentStyle::Prefixed = ident_style {
            buf.push_str(&ident);
            buf.push_str(": ");
        }

        buf.push('{');
        buf.push_str(&ident);
        buf.push_str(&fmt);
        buf.push('}');

        Ok(())
    }
}

impl ToTokens for Output {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

/// See [crate documentation](https://docs.rs/justerror)
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Error(args: CompilerTokenStream, item: CompilerTokenStream) -> CompilerTokenStream {
    let mut error = parse_macro_input!(item as DeriveInput);

    let error_args = if !args.is_empty() {
        parse_macro_input!(args as ErrorArgs)
    } else {
        ErrorArgs::default()
    };

    match &mut error.data {
        Data::Enum(data) => {
            for variant in &mut data.variants {
                let mut variant_error_attr: Option<(usize, ErrorArgs)> = None;

                for (idx, attr) in &mut variant.attrs.iter().enumerate() {
                    if attr.path.is_ident(ERROR_ATTR) {
                        let error_args = match attr.parse_args::<ErrorArgs>() {
                            Ok(args) => args,
                            Err(err) => return err.into_compile_error().into(),
                        };
                        variant_error_attr = Some((idx, error_args));
                    }
                }

                let (variant_error_attr_idx, variant_error_args) = match variant_error_attr {
                    Some((idx, args)) => (Some(idx), Some(args)),
                    None => (None, None),
                };

                if let Some(idx) = variant_error_attr_idx {
                    variant.attrs.remove(idx);
                }

                let mut output = Output::new();

                output.push_title(&error.ident, Some(&variant.ident));

                match (&error_args.desc, &variant_error_args) {
                    (
                        Some(error_desc),
                        Some(ErrorArgs {
                            desc: Some(variant_desc),
                            fmt: _,
                        }),
                    ) => {
                        output.push_desc(Some(&error.ident), error_desc);
                        output.push_desc(Some(&variant.ident), variant_desc);
                    }
                    (Some(error_desc), Some(ErrorArgs { desc: None, fmt: _ }) | None) => {
                        output.push_desc(None, error_desc);
                    }
                    (
                        None,
                        Some(ErrorArgs {
                            desc: Some(variant_desc),
                            fmt: _,
                        }),
                    ) => {
                        output.push_desc(None, variant_desc);
                    }
                    (None, Some(ErrorArgs { desc: None, fmt: _ }) | None) => (),
                };

                if let Err(err) =
                    output.push_fields(&mut variant.fields, &error_args, &variant_error_args)
                {
                    return err.into();
                }

                variant.attrs.push(parse_quote!(#[error(#output)]));
            }
        }
        Data::Struct(data) => {
            let mut output = Output::new();

            output.push_title(&error.ident, None);

            if let Some(desc) = &error_args.desc {
                output.push_desc(None, desc);
            }

            if let Err(err) = output.push_fields(&mut data.fields, &error_args, &None) {
                {
                    return err.into();
                }
            }

            error.attrs.push(parse_quote!(#[error(#output)]));
        }
        Data::Union(_) => {
            return SyntaxError::new_spanned(
                error,
                "Untagged unions are not supported by the Error macro.",
            )
            .to_compile_error()
            .into()
        }
    }

    quote! {
      #[derive(thiserror::Error, Debug)]
      #error
    }
    .into()
}

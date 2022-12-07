#[macro_use]
extern crate justerror;

use indoc::indoc;

#[Error]
enum EnumError {
    Foo,
    Bar { a: &'static str, b: usize },
    Baz(&'static str),
    Qux(#[fmt(debug)] Vec<&'static str>, usize),
}

#[Error(desc = "My enum error", fmt = debug)]
enum EnumErrorWithArgs {
    #[error(desc = "Foo error")]
    Foo,
    #[error(desc = "Bar error", fmt = display)]
    Bar {
        a: &'static str,
        #[fmt("05")]
        b: usize,
    },
    #[error(desc = "Baz error")]
    Baz(&'static str),
    Qux(Vec<&'static str>, usize),
}

#[Error(desc = "My struct error")]
struct MultipleNamedFieldsStructError {
    a: &'static str,
    #[fmt(">5")]
    b: usize,
}

#[Error]
struct SingleUnnamedFieldStructError(&'static str);

#[test]
fn it_formats_enum_error_without_fields() {
    let actual = format!("{}", EnumError::Foo);
    let expected = "EnumError::Foo";

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_enum_error_with_named_fields() {
    let actual = format!("{}", EnumError::Bar { a: "A", b: 42 });
    let expected = indoc! {r#"
        EnumError::Bar
        === DEBUG DATA:
        a: A
        b: 42"#};

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_enum_error_with_single_unnamed_field() {
    let actual = format!("{}", EnumError::Baz("Oh no"));
    let expected = indoc! {r#"
        EnumError::Baz
        === DEBUG DATA:
        Oh no"#};

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_enum_error_with_multiple_unnamed_fields() {
    let actual = format!("{}", EnumError::Qux(vec!["One", "Two"], 42));
    let expected = indoc! {r#"
        EnumError::Qux
        === DEBUG DATA:
        0: [
            "One",
            "Two",
        ]
        1: 42"#};

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_enum_error_with_args_without_fields() {
    let actual = format!("{}", EnumErrorWithArgs::Foo);
    let expected = indoc! {r#"
        EnumErrorWithArgs::Foo
        EnumErrorWithArgs: My enum error
        Foo: Foo error"#};

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_enum_error_with_args_with_field_with_custom_format() {
    let actual = format!("{}", EnumErrorWithArgs::Bar { a: "A", b: 42 });
    let expected = indoc! {r#"
        EnumErrorWithArgs::Bar
        EnumErrorWithArgs: My enum error
        Bar: Bar error
        === DEBUG DATA:
        a: A
        b: 00042"#};

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_enum_error_with_args_with_single_unnamed_field() {
    let actual = format!("{}", EnumErrorWithArgs::Baz("Oh no"));
    let expected = indoc! {r#"
        EnumErrorWithArgs::Baz
        EnumErrorWithArgs: My enum error
        Baz: Baz error
        === DEBUG DATA:
        "Oh no""#};

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_enum_error_with_args_with_field_using_root_format() {
    let actual = format!("{}", EnumErrorWithArgs::Qux(vec!["One", "Two"], 42));
    let expected = indoc! {r#"
        EnumErrorWithArgs::Qux
        My enum error
        === DEBUG DATA:
        0: [
            "One",
            "Two",
        ]
        1: 42"#};

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_multiple_named_fields_struct_error() {
    let actual = format!("{}", MultipleNamedFieldsStructError { a: "A", b: 7 });
    let expected = indoc! {r#"
        MultipleNamedFieldsStructError
        My struct error
        === DEBUG DATA:
        a: A
        b:     7"#};

    assert_eq!(actual, expected);
}

#[test]
fn it_formats_single_unnamed_field_struct_error() {
    let actual = format!("{}", SingleUnnamedFieldStructError("Oh no"));
    let expected = indoc! {r#"
        SingleUnnamedFieldStructError
        === DEBUG DATA:
        Oh no"#};

    assert_eq!(actual, expected);
}

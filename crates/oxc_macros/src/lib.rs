use syn::{parse_macro_input, Item};

mod ast_node;
mod declare_all_lint_rules;
mod declare_oxc_lint;
mod ser_raw;

/// Macro used to declare an oxc lint rule
///
/// Every lint declaration consists of 2 parts:
///
/// 1. The documentation
/// 2. The lint's struct
///
/// # Example
///
/// ```
/// use oxc_macros::declare_oxc_lint;
///
/// declare_oxc_lint! {
///     /// ### What it does
///     /// Checks for usage of the `debugger` statement
///     ///
///     /// ### Why is this bad?
///     /// `debugger` statements do not affect functionality when a debugger isn't attached.
///     /// They're most commonly an accidental debugging leftover.
///     ///
///     ///
///     /// ### Example
///     /// ```javascript
///     /// const data = await getData();
///     /// const result = complexCalculation(data);
///     /// debugger;
///     /// ```
///     ///
///     /// ```
///     pub struct NoDebugger
/// }
/// ```
#[proc_macro]
pub fn declare_oxc_lint(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let metadata = parse_macro_input!(input as declare_oxc_lint::LintRuleMeta);

    declare_oxc_lint::declare_oxc_lint(metadata).into()
}

/// Same as `declare_oxc_lint`, but doesn't do imports.
/// Enables multiple usages in a single file.
#[proc_macro]
pub fn declare_oxc_lint_test(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut metadata = parse_macro_input!(input as declare_oxc_lint::LintRuleMeta);
    metadata.used_in_test = true;

    declare_oxc_lint::declare_oxc_lint(metadata).into()
}

#[proc_macro]
pub fn declare_all_lint_rules(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let metadata = parse_macro_input!(input as declare_all_lint_rules::AllLintRulesMeta);

    declare_all_lint_rules::declare_all_lint_rules(metadata).into()
}

/// Attribute for AST nodes
#[proc_macro_attribute]
pub fn ast_node(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Item);
    ast_node::ast_node(&input).into()
}

/// Attribute for AST nodes which can be serialized in "raw" format
#[proc_macro_attribute]
pub fn ser_raw(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as Item);
    ser_raw::ser_raw(&mut input).into()
}

/// Dummy derive macro which doesn't actually derive anything.
/// `ast_node` macro "derives" this.
/// Purpose is allowing use of `#[serde]` attributes on structs/enums without deriving
/// `serde::Serialize`, so can remove a lot of boilerplate - `#[serde(flatten)]`
/// instead of `#[cfg_attr(any(feature = "serde", feature = "raw"), serde(flatten)]`.
#[proc_macro_derive(SerdeAttrs, attributes(serde))]
pub fn serde_attrs(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}

use cow_utils::CowUtils;
use oxc_ast::{
    ast::{match_member_expression, CallExpression, ChainElement, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{cmp::ContentEq, GetSpan, Span};
use phf::phf_set;

use crate::{
    ast_util::{self, is_method_call},
    context::LintContext,
    rule::Rule,
    AstNode,
};

fn eslint_prefer_spread_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Require spread operators instead of .apply()").with_label(span)
}

fn unicorn_prefer_spread_diagnostic(span: Span, bad_method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer the spread operator (`...`) over {bad_method}"))
        .with_help("The spread operator (`...`) is more concise and readable.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSpread;

declare_oxc_lint!(
    /// This rule is combined 2 rules from `eslint:prefer-spread` and `unicorn:prefer-spread`.
    ///
    /// ## original eslint:prefer-spread
    ///
    /// ### What it does
    ///
    /// Require spread operators instead of .apply()
    ///
    /// ### Why is this bad?
    ///
    /// Before ES2015, one must use Function.prototype.apply() to call variadic functions.
    /// ```javascript
    /// var args = [1, 2, 3, 4];
    /// Math.max.apply(Math, args);
    /// ```
    ///
    /// In ES2015, one can use spread syntax to call variadic functions.
    /// ```javascript
    /// var args = [1, 2, 3, 4];
    /// Math.max(...args);
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo.apply(undefined, args);
    /// foo.apply(null, args);
    /// obj.foo.apply(obj, args);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // Using spread syntax
    /// foo(...args);
    /// obj.foo(...args);
    ///
    /// // The `this` binding is different.
    /// foo.apply(obj, args);
    /// obj.foo.apply(null, args);
    /// obj.foo.apply(otherObj, args);
    ///
    /// // The argument list is not variadic.
    /// // Those are warned by the `no-useless-call` rule.
    /// foo.apply(undefined, [1, 2, 3]);
    /// foo.apply(null, [1, 2, 3]);
    /// obj.foo.apply(obj, [1, 2, 3]);
    /// ```
    ///
    /// ## unicorn:prefer-spread
    ///
    /// ### What it does
    ///
    /// Enforces the use of [the spread operator (`...`)](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax) over outdated patterns.
    ///
    /// ### Why is this bad?
    ///
    /// Using the spread operator is more concise and readable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = Array.from(set);
    /// const foo = Array.from(new Set([1, 2]));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// [...set].map(() => {});
    /// Array.from(...argumentsArray);
    /// ```
    PreferSpread,
    style,
    conditional_fix
);

impl Rule for PreferSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        check_eslint_prefer_spread(call_expr, ctx);

        check_unicorn_prefer_spread(call_expr, ctx);
    }
}

fn check_eslint_prefer_spread(call_expr: &CallExpression, ctx: &LintContext) {
    if !is_method_call(call_expr, None, Some(&["apply"]), Some(2), Some(2)) {
        return;
    }

    let callee = call_expr.callee.without_parentheses();

    let callee = match callee {
        match_member_expression!(Expression) => callee.to_member_expression(),
        Expression::ChainExpression(chain) => match chain.expression {
            match_member_expression!(ChainElement) => chain.expression.to_member_expression(),
            _ => return,
        },
        _ => return,
    };

    let args = &call_expr.arguments;

    let Some(args0) = args[0].as_expression() else {
        return;
    };

    if args[1].is_spread() {
        return;
    }

    if let Some(Expression::ArrayExpression(_)) = args[1].as_expression() {
        return;
    }

    let applied = callee.object().without_parentheses();

    if args0.is_null_or_undefined() {
        if !matches!(applied, Expression::Identifier(_)) {
            return;
        }
    } else if let Some(applied) = as_member_expression_without_chain_expression(applied) {
        let applied_object = applied.object().without_parentheses();

        if let Some(args0) = as_member_expression_without_chain_expression(args0) {
            let Some(applied_object) =
                as_member_expression_without_chain_expression(applied_object)
            else {
                return;
            };

            if applied_object.content_ne(args0) {
                return;
            }
        } else if applied_object.content_ne(args0) {
            return;
        }
    } else {
        return;
    }

    ctx.diagnostic(eslint_prefer_spread_diagnostic(call_expr.span));
}

fn as_member_expression_without_chain_expression<'a>(
    expr: &'a Expression,
) -> Option<&'a MemberExpression<'a>> {
    match expr {
        Expression::ChainExpression(chain_expr) => match chain_expr.expression {
            match_member_expression!(ChainElement) => chain_expr.expression.as_member_expression(),
            _ => None,
        },
        match_member_expression!(Expression) => expr.as_member_expression(),
        _ => None,
    }
}

fn check_unicorn_prefer_spread(call_expr: &CallExpression, ctx: &LintContext) {
    let Some(member_expr) = call_expr.callee.without_parentheses().as_member_expression() else {
        return;
    };

    let Some(static_property_name) = member_expr.static_property_name() else {
        return;
    };

    match static_property_name {
        // `Array.from()`
        "from" => {
            if call_expr.arguments.len() != 1 || member_expr.is_computed() {
                return;
            }

            let Some(expr) = call_expr.arguments[0].as_expression() else {
                return;
            };

            if matches!(expr.without_parentheses(), Expression::ObjectExpression(_)) {
                return;
            }

            let Expression::Identifier(ident) = member_expr.object().without_parentheses() else {
                return;
            };

            if ident.name != "Array" {
                return;
            }

            ctx.diagnostic(unicorn_prefer_spread_diagnostic(call_expr.span, "Array.from()"));
        }
        // `array.concat()`
        "concat" => {
            if is_not_array(member_expr.object().without_parentheses(), ctx) {
                return;
            }

            ctx.diagnostic(unicorn_prefer_spread_diagnostic(call_expr.span, "array.concat()"));
        }
        // `array.slice()`
        "slice" => {
            if call_expr.arguments.len() > 1 {
                return;
            }

            let member_expr_obj = member_expr.object().without_parentheses();

            if matches!(
                member_expr_obj,
                Expression::ArrayExpression(_) | Expression::ThisExpression(_)
            ) {
                return;
            }

            if let Expression::Identifier(ident) = member_expr_obj {
                if IGNORED_SLICE_CALLEE.contains(ident.name.as_str()) {
                    return;
                }
            }

            if let Some(first_arg) = call_expr.arguments.first() {
                let Some(first_arg) = first_arg.as_expression() else {
                    return;
                };

                if let Expression::NumericLiteral(num_lit) = first_arg.without_parentheses() {
                    if num_lit.value != 0.0 {
                        return;
                    }
                } else {
                    return;
                }
            }

            ctx.diagnostic(unicorn_prefer_spread_diagnostic(call_expr.span, "array.slice()"));
        }
        // `array.toSpliced()`
        "toSpliced" => {
            if call_expr.arguments.len() != 0 {
                return;
            }

            if matches!(member_expr.object().without_parentheses(), Expression::ArrayExpression(_))
            {
                return;
            }

            ctx.diagnostic(unicorn_prefer_spread_diagnostic(call_expr.span, "array.toSpliced()"));
        }
        // `string.split()`
        "split" => {
            if call_expr.arguments.len() != 1 {
                return;
            }

            let Some(expr) = call_expr.arguments[0].as_expression() else {
                return;
            };

            let Expression::StringLiteral(string_lit) = expr.without_parentheses() else {
                return;
            };

            if string_lit.value != "" {
                return;
            }

            ctx.diagnostic_with_fix(
                unicorn_prefer_spread_diagnostic(call_expr.span, "string.split()"),
                |fixer| {
                    let callee_obj = member_expr.object().without_parentheses();

                    fixer.replace(
                        call_expr.span,
                        format!("[...{}]", callee_obj.span().source_text(ctx.source_text())),
                    )
                },
            );
        }

        _ => {}
    }
}

const IGNORED_SLICE_CALLEE: phf::Set<&'static str> = phf_set! {
    "arrayBuffer",
    "blob",
    "buffer",
    "file",
    "this",
};

fn is_not_array(expr: &Expression, ctx: &LintContext) -> bool {
    if matches!(
        expr.without_parentheses(),
        Expression::TemplateLiteral(_) | Expression::BinaryExpression(_)
    ) {
        return true;
    }

    if expr.is_literal() {
        return true;
    }

    if let Expression::CallExpression(call_expr) = expr {
        if let Some(member_expr) = call_expr.callee.without_parentheses().as_member_expression() {
            if Some("join") == member_expr.static_property_name() && call_expr.arguments.len() < 2 {
                return true;
            }

            return false;
        }

        return false;
    }

    let ident = match expr.without_parentheses() {
        Expression::Identifier(ident) => {
            if let Some(symbol_id) = ast_util::get_symbol_id_of_variable(ident, ctx) {
                let symbol_table = ctx.semantic().symbols();

                let node = ctx.nodes().get_node(symbol_table.get_declaration(symbol_id));

                if let AstKind::VariableDeclarator(variable_declarator) = node.kind() {
                    if let Some(ref_expr) = &variable_declarator.init {
                        return is_not_array(ref_expr, ctx);
                    }
                }
            }

            ident.name.as_str()
        }

        expr @ match_member_expression!(Expression) => {
            if let Some(v) = expr.to_member_expression().static_property_name() {
                v
            } else {
                return false;
            }
        }

        _ => return false,
    };

    if ident.starts_with(|c: char| c.is_ascii_uppercase())
        && ident.cow_to_ascii_uppercase() != ident
    {
        return true;
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        /* Test cases for original eslint:prefer-spread */
        "foo.apply(obj, args);",
        "obj.foo.apply(null, args);",
        "obj.foo.apply(otherObj, args);",
        "a.b(x, y).c.foo.apply(a.b(x, z).c, args);",
        "a.b.foo.apply(a.b.c, args);",
        "foo.apply(undefined, [1, 2]);",
        "foo.apply(null, [1, 2]);",
        "obj.foo.apply(obj, [1, 2]);",
        "var apply; foo[apply](null, args);",
        "foo.apply();",
        "obj.foo.apply();",
        "obj.foo.apply(obj, ...args)",
        "(a?.b).c.foo.apply(a?.b.c, args);",
        "a?.b.c.foo.apply((a?.b).c, args);",
        "class C { #apply; foo() { foo.#apply(undefined, args); } }",
        /* Test cases for unicorn:prefer-spread */
        r"[...set].map(() => {});",
        r"Int8Array.from(set);",
        r"Uint8Array.from(set);",
        r"Uint8ClampedArray.from(set);",
        r"Int16Array.from(set);",
        r"Uint16Array.from(set);",
        r"Int32Array.from(set);",
        r"Uint32Array.from(set);",
        r"Float32Array.from(set);",
        r"Float64Array.from(set);",
        r"BigInt64Array.from(set);",
        r"BigUint64Array.from(set);",
        r"new Array.from(foo);",
        r"from(foo);",
        r#"Array["from"](foo);"#,
        r"Array[from](foo);",
        r"Array.foo(foo);",
        r"foo.from(foo);",
        r"lib.Array.from(foo);",
        r"Array.from();",
        r"Array.from(foo, mapFn, thisArg, extra);",
        r"Array.from(...argumentsArray);",
        r"Array.from(set, mapFn).reduce(() => {});",
        r"Array.from(set, mapFn, thisArg).reduce(() => {});",
        r"Array.from(set, () => {}, thisArg).reduce(() => {});",
        r"Array.from({length: 10});",
        r"new Array.concat(1)",
        r"concat(1)",
        r"array[concat](1)",
        r#""foo".concat("bar")"#,
        r#"`${foo}`.concat("bar")"#,
        r"const bufA = Buffer.concat([buf1, buf2, buf3], totalLength);",
        r"Foo.concat(1)",
        r"FooBar.concat(1)",
        r"global.Buffer.concat([])",
        r#"["1", "2"].join(",").concat("...")"#,
        r#"foo.join(",").concat("...")"#,
        r"foo.join().concat(bar)",
        // r#"(a + b).concat(c)"#,
        r"new Array.slice()",
        r"slice()",
        r"array[slice]()",
        r"array.slice",
        r"array.slice(1)",
        r"array.slice(...[])",
        r"array.slice(...[0])",
        r"array.slice(0 + 0)",
        r#"array.slice("")"#,
        r"array.slice(null)",
        r"const ZERO = 0;array.slice(ZERO)",
        r"array.slice(0, array.length)",
        r"array.slice(0, 0)",
        r"array.notSlice()",
        r"[...foo].slice()",
        r"[foo].slice()",
        r"arrayBuffer.slice()",
        r"blob.slice()",
        r"buffer.slice()",
        r"file.slice()",
        r"class A {foo() {this.slice()}}",
        r"new Array.toSpliced()",
        r"toSpliced()",
        r"array[toSpliced]()",
        r"array.toSpliced",
        r"array.toSpliced(0)",
        r"array.toSpliced(...[])",
        r"array.toSpliced(...[0])",
        r"array.toSpliced(0 + 0)",
        r#"array.toSpliced("")"#,
        r"array.toSpliced(null)",
        r"const ZERO = 0;array.toSpliced(0, ZERO)",
        r"array.toSpliced(0, array.length)",
        r"array.toSpliced(0, 0)",
        r"array.notToSpliced()",
        r"[...foo].toSpliced()",
        r"[foo].toSpliced()",
        r"array.toSpliced(100, 0)",
        r"array.toSpliced(-1, 0)",
        r#"new foo.split("")"#,
        r#"split("")"#,
        r#"string[split]("")"#,
        r"string.split",
        r"string.split(1)",
        r#"string.split(..."")"#,
        r#"string.split(...[""])"#,
        r#"string.split("" + "")"#,
        r"string.split(0)",
        r"string.split(false)",
        r"string.split(undefined)",
        r"string.split(0n)",
        r"string.split(null)",
        r#"string.split(/""/)"#,
        r"string.split(``)",
        r#"const EMPTY_STRING = ""; string.split(EMPTY_STRING)"#,
        r#"string.split("", limit)"#,
        r#""".split(string)"#,
        r"string.split()",
        r#"string.notSplit("")"#,
        r#"const x = "foo"; x.concat(x);"#,
        r#"const y = "foo"; const x = y; x.concat(x);"#,
    ];

    let fail = vec![
        /* Test cases for original eslint:prefer-spread */
        "foo.apply(undefined, args);",
        "foo.apply(void 0, args);",
        "foo.apply(null, args);",
        "obj.foo.apply(obj, args);",
        "a.b.c.foo.apply(a.b.c, args);",
        "a.b(x, y).c.foo.apply(a.b(x, y).c, args);",
        "[].concat.apply([ ], args);",
        "[].concat.apply([
			/*empty*/
			], args);",
        "foo.apply?.(undefined, args);",
        "foo?.apply(undefined, args);",
        "foo?.apply?.(undefined, args);",
        "(foo?.apply)(undefined, args);",
        "(foo?.apply)?.(undefined, args);",
        "(obj?.foo).apply(obj, args);",
        "a?.b.c.foo.apply(a?.b.c, args);",
        "(a?.b.c).foo.apply(a?.b.c, args);",
        "(a?.b).c.foo.apply((a?.b).c, args);",
        "class C { #foo; foo() { obj.#foo.apply(obj, args); } }",
        /* Test cases for unicorn:prefer-spread */
        r"const x = Array.from(set);",
        r"Array.from(set).map(() => {});",
        r"Array.from(new Set([1, 2])).map(() => {});",
        r#"Array.from(document.querySelectorAll("*")).map(() => {});"#,
        r"const foo = `${Array.from(arrayLike)}`",
        r"(Array).from(foo)",
        r"(Array.from)(foo)",
        r"((Array).from)(foo)",
        r"(Array).from((0, foo))",
        r"(Array.from)((0, foo))",
        r"((Array).from)((0, foo))",
        r"Array.from(a ? b : c)",
        r"Array.from([...a, ...b], )",
        r"Array.from([1])",
        r"Array.from([...a, ...b])",
        r"/* 1 */ Array /* 2 */ .from /* 3 */ ( /* 4 */ a /* 5 */,)",
        r"[1].concat(2)",
        r"[1].concat([2, 3])",
        r"[1].concat(2,)",
        r"[1].concat([2, ...bar],)",
        r"[1,].concat(2)",
        r"[1,].concat([2, 3])",
        r"[1,].concat(2,)",
        r"[1,].concat([2, 3],)",
        r"(( (( (( [1,] )).concat ))( (([2, 3])) ,) ))",
        r"(( (( (( [1,] )).concat ))( (([2, 3])) , bar ) ))",
        r"foo.concat(2)",
        r"foo.concat([2, 3])",
        r"foo.concat(2,)",
        r"foo.concat([2, 3],)",
        r"(( (( ((foo)).concat ))( (([2, 3])) ,) ))",
        r"(( (( ((foo)).concat ))( (([2, 3])) , bar ) ))",
        r"const foo = foo.concat(2)",
        r"const foo = () => foo.concat(2)",
        r"foo.concat([bar])",
        r"foo.concat(bar)",
        r"Array.from(set).concat([2, 3])",
        r"foo.concat([2, 3]).concat(4)",
        r#"string.concat("bar")"#,
        r"foo.concat(2, 3)",
        r"foo.concat(2, bar)",
        r"[...foo, 2].concat(bar)",
        r"let sortedScores = scores.concat().sort((a, b) => b[0] - a[0]);",
        r"foo.concat(bar, 2, 3)",
        r"foo.concat(bar, 2, 3, baz)",
        r"async function a() {return [].concat(await bar)}",
        r"async function a() {return [].concat(((await bar)))}",
        r"foo.concat((0, 1))",
        r"async function a() {return (await bar).concat(1)}",
        r"[].concat(...bar)",
        r"[].concat([,], [])",
        r"[,].concat([,], [,])",
        r"[,].concat([,,], [,])",
        r"[,].concat([,], [,,])",
        r"[1].concat([2,], [3,])",
        r"[1].concat([2,,], [3,,])",
        r"[1,].concat([2,], [3,])",
        r"[1,].concat([2,,], [3,,])",
        r"[].concat([], [])",
        r"[].concat((a.b.c), 2)",
        r"[].concat(a.b(), 2)",
        r"foo.concat(bar, 2, [3, 4], baz, 5, [6, 7])",
        r"foo.concat(bar, 2, 3, ...baz)",
        r"notClass.concat(1)",
        r"_A.concat(1)",
        r"FOO.concat(1)",
        r"A.concat(1)",
        r"Foo.x.concat(1)",
        r"if (test) foo.concat(1)",
        r"if (test) {} else foo.concat(1)",
        r"if (test) {} else foo.concat(1)",
        r"for (;;) foo.concat(1)",
        r"for (a in b) foo.concat(1)",
        r"for (a in b) foo.concat(1)",
        r"for (const a of b) foo.concat(1)",
        r"while (test) foo.concat(1)",
        r"do foo.concat(1); while (test)",
        r"with (foo) foo.concat(1)",
        r#"foo.join(foo, bar).concat("...")"#,
        r"array.slice()",
        r"array.slice().slice()",
        r"array.slice(1).slice()",
        r"array.slice().slice(1)",
        r"const copy = array.slice()",
        r"(( (( (( array )).slice ))() ))",
        r#""".slice()"#,
        r"new Uint8Array([10, 20, 30, 40, 50]).slice()",
        r"array.slice(0)",
        r"array.slice(0b0)",
        r"array.slice(0.00)",
        r"array.slice(0.00, )",
        r"array.toSpliced()",
        r"array.toSpliced().toSpliced()",
        r"const copy = array.toSpliced()",
        r"(( (( (( array )).toSpliced ))() ))",
        r#""".toSpliced()"#,
        r"new Uint8Array([10, 20, 30, 40, 50]).toSpliced()",
        r#""string".split("")"#,
        r#""string".split('')"#,
        r#"unknown.split("")"#,
        r#"const characters = "string".split("")"#,
        r#"(( (( (( "string" )).split ))( (("")) ) ))"#,
        r#"unknown.split("")"#,
        r#""🦄".split("")"#,
        r#"const {length} = "🦄".split("")"#,
    ];

    let expect_fix = vec![
        /* Test cases for unicorn:prefer-spread */
        // `Array.from()`
        // `array.slice()`
        // `array.toSpliced()`
        // `string.split()`
        (r#""🦄".split("")"#, r#"[..."🦄"]"#, None),
        (r#""foo bar baz".split("")"#, r#"[..."foo bar baz"]"#, None),
    ];

    Tester::new(PreferSpread::NAME, PreferSpread::CATEGORY, pass, fail)
        .expect_fix(expect_fix)
        .test_and_snapshot();
}
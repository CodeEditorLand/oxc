use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstKind};

use crate::{array, format::print::statement, hardline, indent, ir::Doc, text, Format, Prettier};

pub fn print_block<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
    directives: Option<&[Directive<'a>]>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    parts.push(text!("{"));
    if let Some(doc) = print_block_body(p, stmts, directives) {
        parts.push({
            let mut parts = Vec::new_in(p.allocator);
            parts.extend(hardline!());
            parts.push(doc);
            indent!(p, parts)
        });
        parts.extend(hardline!());
    } else {
        let parent = p.parent_kind();

        let parent_parent = p.parent_parent_kind();

        if (parent_parent.is_none()
            || parent_parent.is_some_and(|p| !matches!(p, AstKind::ObjectProperty(_))))
            && !(matches!(
                parent,
                AstKind::FunctionBody(_)
                    | AstKind::ArrowFunctionExpression(_)
                    | AstKind::ObjectExpression(_)
                    | AstKind::Function(_)
                    | AstKind::ForStatement(_)
                    | AstKind::WhileStatement(_)
                    | AstKind::DoWhileStatement(_)
                    | AstKind::MethodDefinition(_)
                    | AstKind::PropertyDefinition(_)
            ) || (matches!(parent, AstKind::CatchClause(_))
                && !matches!(p.parent_parent_kind(), Some(AstKind::TryStatement(stmt)) if stmt.finalizer.is_some()))
                || matches!(p.current_kind(), AstKind::StaticBlock(_)))
        {
            parts.extend(hardline!());
        }
    }
    parts.push(text!("}"));

    array!(p, parts)
}

pub fn print_block_body<'a>(
    p: &mut Prettier<'a>,
    stmts: &[Statement<'a>],
    directives: Option<&[Directive<'a>]>,
) -> Option<Doc<'a>> {
    let has_directives = directives.is_some_and(|directives| !directives.is_empty());

    let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));

    if !has_body && !has_directives {
        return None;
    }

    let mut parts = Vec::new_in(p.allocator);

    if has_directives {
        if let Some(directives) = directives {
            parts.extend(directives.iter().map(|d| d.format(p)));
        }
    }

    if has_body {
        parts.extend(statement::print_statement_sequence(p, stmts));
    }

    Some(array!(p, parts))
}
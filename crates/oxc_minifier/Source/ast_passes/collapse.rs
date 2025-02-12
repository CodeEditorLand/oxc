use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{CompressOptions, CompressorPass};

/// Collapse variable declarations (TODO: and assignments).
///
/// `var a; var b = 1; var c = 2` => `var a, b = 1; c = 2`
/// TODO: `a = null; b = null;` => `a = b = null`
pub struct Collapse {
	options:CompressOptions,
}

impl<'a> CompressorPass<'a> for Collapse {}

impl<'a> Traverse<'a> for Collapse {
	fn enter_statements(&mut self, stmts:&mut Vec<'a, Statement<'a>>, ctx:&mut TraverseCtx<'a>) {
		if self.options.join_vars {
			self.join_vars(stmts, ctx);
		}
	}
}

impl<'a> Collapse {
	pub fn new(options:CompressOptions) -> Self { Self { options } }

	/// Join consecutive var statements
	fn join_vars(&self, stmts:&mut Vec<'a, Statement<'a>>, ctx:&mut TraverseCtx<'a>) {
		// Collect all the consecutive ranges that contain joinable vars.
		// This is required because Rust prevents in-place vec mutation.
		let mut ranges = vec![];

		let mut range = 0..0;

		let mut i = 1usize;

		let mut capacity = 0usize;

		for window in stmts.windows(2) {
			let [prev, cur] = window else { unreachable!() };

			if let (
				Statement::VariableDeclaration(cur_decl),
				Statement::VariableDeclaration(prev_decl),
			) = (cur, prev)
			{
				// Do not join `require` calls for cjs-module-lexer.
				if cur_decl
					.declarations
					.first()
					.and_then(|d| d.init.as_ref())
					.is_some_and(Expression::is_require_call)
				{
					break;
				}

				if cur_decl.kind == prev_decl.kind {
					if i - 1 != range.end {
						range.start = i - 1;
					}

					range.end = i + 1;
				}
			}

			if (range.end != i || i == stmts.len() - 1) && range.start < range.end {
				capacity += range.end - range.start - 1;

				ranges.push(range.clone());

				range = 0..0;
			}

			i += 1;
		}

		if ranges.is_empty() {
			return;
		}

		// Reconstruct the stmts array by joining consecutive ranges
		let mut new_stmts = ctx.ast.vec_with_capacity(stmts.len() - capacity);

		for (i, stmt) in stmts.drain(..).enumerate() {
			if i > 0 && ranges.iter().any(|range| range.contains(&(i - 1)) && range.contains(&i)) {
				if let Statement::VariableDeclaration(prev_decl) = new_stmts.last_mut().unwrap() {
					if let Statement::VariableDeclaration(mut cur_decl) = stmt {
						prev_decl.declarations.append(&mut cur_decl.declarations);
					}
				}
			} else {
				new_stmts.push(stmt);
			}
		}
		*stmts = new_stmts;
	}
}

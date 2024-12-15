use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_semantic::{ScopeTree, SemanticBuilder, SymbolTable};
use oxc_traverse::ReusableTraverseCtx;

use crate::{
	CompressOptions,
	CompressorPass,
	ast_passes::{
		CollapsePass,
		LatePeepholeOptimizations,
		PeepholeFoldConstants,
		PeepholeOptimizations,
		PeepholeRemoveDeadCode,
		RemoveSyntax,
	},
};

pub struct Compressor<'a> {
	allocator:&'a Allocator,
	options:CompressOptions,
}

impl<'a> Compressor<'a> {
	pub fn new(allocator:&'a Allocator, options:CompressOptions) -> Self {
		Self { allocator, options }
	}

	pub fn build(self, program:&mut Program<'a>) {
		let (symbols, scopes) = SemanticBuilder::new()
			.build(program)
			.semantic
			.into_symbol_table_and_scope_tree();

		self.build_with_symbols_and_scopes(symbols, scopes, program);
	}

	pub fn build_with_symbols_and_scopes(
		self,
		symbols:SymbolTable,
		scopes:ScopeTree,
		program:&mut Program<'a>,
	) {
		let mut ctx = ReusableTraverseCtx::new(scopes, symbols, self.allocator);
		RemoveSyntax::new(self.options).build(program, &mut ctx);

		if self.options.dead_code_elimination {
			Self::dead_code_elimination(program, &mut ctx);

			return;
		}

		PeepholeOptimizations::new().build(program, &mut ctx);
		CollapsePass::new().build(program, &mut ctx);
		LatePeepholeOptimizations::new().run_in_loop(program, &mut ctx);
		PeepholeOptimizations::new().build(program, &mut ctx);
	}

	fn dead_code_elimination(program:&mut Program<'a>, ctx:&mut ReusableTraverseCtx<'a>) {
		PeepholeFoldConstants::new().build(program, ctx);
		PeepholeRemoveDeadCode::new().build(program, ctx);
	}
}

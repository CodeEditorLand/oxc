use std::sync::Arc;

use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, VisitMut, ast::*, visit::walk_mut};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::identifier::is_identifier_name;

/// Configuration for [ReplaceGlobalDefines].
///
/// Due to the usage of an arena allocator, the constructor will parse once for
/// grammatical errors, and does not save the constructed expression.
///
/// The data is stored in an `Arc` so this can be shared across threads.
#[derive(Debug, Clone)]
pub struct ReplaceGlobalDefinesConfig(Arc<ReplaceGlobalDefinesConfigImpl>);

#[derive(Debug)]
struct ReplaceGlobalDefinesConfigImpl {
	identifier_defines:Vec<(/* key */ String, /* value */ String)>,
	// TODO: dot defines
}

impl ReplaceGlobalDefinesConfig {
	/// # Errors
	///
	/// * key is not an identifier
	/// * value has a syntax error
	pub fn new<S:AsRef<str>>(defines:&[(S, S)]) -> Result<Self, Vec<OxcDiagnostic>> {
		let allocator = Allocator::default();

		let mut identifier_defines = vec![];

		for (key, value) in defines {
			let key = key.as_ref();

			let value = value.as_ref();

			Self::check_key(key)?;

			Self::check_value(&allocator, value)?;

			identifier_defines.push((key.to_string(), value.to_string()));
		}

		Ok(Self(Arc::new(ReplaceGlobalDefinesConfigImpl { identifier_defines })))
	}

	fn check_key(key:&str) -> Result<(), Vec<OxcDiagnostic>> {
		if !is_identifier_name(key) {
			return Err(vec![OxcDiagnostic::error(format!("`{key}` is not an identifier."))]);
		}

		Ok(())
	}

	fn check_value(allocator:&Allocator, source_text:&str) -> Result<(), Vec<OxcDiagnostic>> {
		Parser::new(allocator, source_text, SourceType::default()).parse_expression()?;

		Ok(())
	}
}

/// Replace Global Defines.
///
/// References:
///
/// * <https://esbuild.github.io/api/#define>
/// * <https://github.com/terser/terser?tab=readme-ov-file#conditional-compilation>
pub struct ReplaceGlobalDefines<'a> {
	ast:AstBuilder<'a>,
	config:ReplaceGlobalDefinesConfig,
}

impl<'a> ReplaceGlobalDefines<'a> {
	pub fn new(allocator:&'a Allocator, config:ReplaceGlobalDefinesConfig) -> Self {
		Self { ast:AstBuilder::new(allocator), config }
	}

	pub fn build(&mut self, program:&mut Program<'a>) { self.visit_program(program); }

	// Construct a new expression because we don't have ast clone right now.
	fn parse_value(&self, source_text:&str) -> Expression<'a> {
		// Allocate the string lazily because replacement happens rarely.
		let source_text = self.ast.allocator.alloc(source_text.to_string());
		// Unwrapping here, it should already be checked by
		// [ReplaceGlobalDefinesConfig::new].
		Parser::new(self.ast.allocator, source_text, SourceType::default())
			.parse_expression()
			.unwrap()
	}

	fn replace_identifier_defines(&self, expr:&mut Expression<'a>) {
		for (key, value) in &self.config.0.identifier_defines {
			if let Expression::Identifier(ident) = expr {
				if ident.name.as_str() == key {
					let value = self.parse_value(value);
					*expr = value;

					break;
				}
			}
		}
	}
}

impl<'a> VisitMut<'a> for ReplaceGlobalDefines<'a> {
	fn visit_expression(&mut self, expr:&mut Expression<'a>) {
		self.replace_identifier_defines(expr);

		walk_mut::walk_expression(self, expr);
	}
}

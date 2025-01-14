//! ESM module lexer
//!
//! * <https://github.com/guybedford/es-module-lexer>

use oxc_ast::{Visit, ast::*, visit::walk};
use oxc_ecmascript::BoundNames;
use oxc_span::{Atom, GetSpan};

#[derive(Debug, Clone)]
pub struct ImportSpecifier<'a> {
	/// Module name
	///
	/// To handle escape sequences in specifier strings, the .n field of
	/// imported specifiers will be provided where possible.
	///
	/// For dynamic import expressions, this field will be empty if not a valid
	/// JS string.
	pub n:Option<Atom<'a>>,

	/// Start of module specifier
	pub s:u32,

	/// End of module specifier
	pub e:u32,

	/// Start of import statement
	pub ss:u32,

	/// End of import statement
	pub se:u32,

	/// Dynamic import / Static import / `import.meta`
	pub d:ImportType,

	/// If this import has an import assertion, this is the start value
	pub a:Option<u32>,

	/// If this import is for types only
	pub t:bool,
}

#[derive(Debug, Clone)]
pub struct ExportSpecifier<'a> {
	/// Exported name
	pub n:Atom<'a>,

	/// Local name, or undefined.
	pub ln:Option<Atom<'a>>,

	/// Start of exported name
	pub s:u32,

	/// End of exported name
	pub e:u32,

	/// Start of local name
	pub ls:Option<u32>,

	/// End of local name
	pub le:Option<u32>,

	/// If this export is for types only
	pub t:bool,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum ImportType {
	/// If this import keyword is a static import
	#[default]
	StaticImport,
	/// If this import is an `export *`
	ExportStar,
	/// If this import keyword is a dynamic import, this is the start value.
	DynamicImport(u32),
	/// If this import keyword is an import.meta expression
	ImportMeta,
}

impl ImportType {
	pub fn as_dynamic_import(&self) -> Option<u32> {
		match self {
			Self::DynamicImport(start) => Some(*start),
			Self::StaticImport | Self::ExportStar | Self::ImportMeta => None,
		}
	}
}

pub struct ModuleLexer<'a> {
	pub imports:Vec<ImportSpecifier<'a>>,

	pub exports:Vec<ExportSpecifier<'a>>,

	/// ESM syntax detection
	///
	/// The use of ESM syntax: import / export statements and `import.meta`
	pub has_module_syntax:bool,

	/// Facade modules that only use import / export syntax
	pub facade:bool,
}

impl<'a> Default for ModuleLexer<'a> {
	fn default() -> Self { Self::new() }
}

impl<'a> ModuleLexer<'a> {
	#[must_use]
	pub fn new() -> Self {
		Self { imports:vec![], exports:vec![], has_module_syntax:false, facade:true }
	}

	#[must_use]
	pub fn build(mut self, program:&Program<'a>) -> Self {
		self.visit_program(program);

		self
	}
}

impl<'a> Visit<'a> for ModuleLexer<'a> {
	fn visit_statement(&mut self, stmt:&Statement<'a>) {
		if self.facade && !stmt.is_module_declaration() && !stmt.is_declaration() {
			self.facade = false;
		}

		walk::walk_statement(self, stmt);
	}

	fn visit_module_declaration(&mut self, decl:&ModuleDeclaration<'a>) {
		if !self.has_module_syntax {
			self.has_module_syntax = true;
		}

		walk::walk_module_declaration(self, decl);
	}

	// import.meta
	fn visit_meta_property(&mut self, prop:&MetaProperty<'a>) {
		if !self.has_module_syntax {
			self.has_module_syntax = true;
		}

		if prop.meta.name == "import" && prop.property.name == "meta" {
			self.imports.push(ImportSpecifier {
				n:None,
				s:prop.span.start,
				e:prop.span.end,
				ss:prop.span.start,
				se:prop.span.end,
				d:ImportType::ImportMeta,
				a:None,
				t:false,
			});
		}

		walk::walk_meta_property(self, prop);
	}

	// import("foo")
	fn visit_import_expression(&mut self, expr:&ImportExpression<'a>) {
		let (source, source_span_start, source_span_end) =
			if let Expression::StringLiteral(s) = &expr.source {
				(Some(s.value.clone()), s.span.start, s.span.end)
			} else {
				let span = expr.source.span();
				(None, span.start, span.end)
			};

		self.imports.push(ImportSpecifier {
			n:source,
			s:source_span_start,
			e:source_span_end,
			ss:expr.span.start,
			se:expr.span.end,
			d:ImportType::DynamicImport(expr.span.start + 6),
			a:expr.arguments.first().map(|e| e.span().start),
			t:false,
		});

		walk::walk_import_expression(self, expr);
	}

	fn visit_ts_import_type(&mut self, impt:&TSImportType<'a>) {
		let (source, source_span) = match &impt.parameter {
			TSType::TSLiteralType(literal_type) => {
				match &literal_type.literal {
					TSLiteral::StringLiteral(s) => (Some(s.value.clone()), s.span()),
					_ => (None, literal_type.span()),
				}
			},
			_ => (None, impt.parameter.span()),
		};

		self.imports.push(ImportSpecifier {
			n:source,
			s:source_span.start,
			e:source_span.end,
			ss:impt.span.start,
			se:impt.span.end,
			d:ImportType::DynamicImport(impt.span.start + 6),
			a:None,
			t:true,
		});
	}

	fn visit_import_declaration(&mut self, decl:&ImportDeclaration<'a>) {
		let assertions = decl
			.with_clause
			.as_ref()
			.filter(|c| c.with_entries.first().is_some_and(|a| a.key.as_atom() == "type"))
			.map(|c| c.span.start);

		self.imports.push(ImportSpecifier {
			n:Some(decl.source.value.clone()),
			s:decl.source.span.start + 1, // +- 1 for removing string quotes
			e:decl.source.span.end - 1,
			ss:decl.span.start,
			se:decl.span.end,
			d:ImportType::StaticImport,
			a:assertions,
			t:decl.import_kind.is_type(),
		});

		walk::walk_import_declaration(self, decl);
	}

	fn visit_export_named_declaration(&mut self, decl:&ExportNamedDeclaration<'a>) {
		if let Some(source) = &decl.source {
			// export { named } from 'foo'
			self.imports.push(ImportSpecifier {
				n:Some(source.value.clone()),
				s:source.span.start + 1,
				e:source.span.end - 1,
				ss:decl.span.start,
				se:decl.span.end,
				d:ImportType::StaticImport,
				a:None,
				t:decl.export_kind.is_type(),
			});
		}

		// export const/let/var/function/class ...
		if let Some(decl) = &decl.declaration {
			if self.facade {
				self.facade = false;
			}

			decl.bound_names(&mut |ident| {
				self.exports.push(ExportSpecifier {
					n:ident.name.clone(),
					ln:Some(ident.name.clone()),
					s:ident.span.start,
					e:ident.span.end,
					ls:None,
					le:None,
					t:false,
				});
			});
		}

		// export { named }

		self.exports.extend(decl.specifiers.iter().map(|s| {
			let (exported_start, exported_end) = match &s.exported {
				ModuleExportName::IdentifierName(ident) => (ident.span.start, ident.span.end),
				ModuleExportName::IdentifierReference(ident) => (ident.span.start, ident.span.end),
				// +1 -1 to remove the string quotes
				ModuleExportName::StringLiteral(s) => (s.span.start + 1, s.span.end - 1),
			};

			ExportSpecifier {
				n:s.exported.name().clone(),
				ln:decl.source.is_none().then(|| s.local.name().clone()),
				s:exported_start,
				e:exported_end,
				ls:Some(s.local.span().start),
				le:Some(s.local.span().end),
				t:decl.export_kind.is_type(),
			}
		}));

		walk::walk_export_named_declaration(self, decl);
	}

	// export default foo
	fn visit_export_default_declaration(&mut self, decl:&ExportDefaultDeclaration<'a>) {
		if self.facade {
			self.facade = false;
		}

		let ln = match &decl.declaration {
			ExportDefaultDeclarationKind::FunctionDeclaration(func) => func.id.as_ref(),
			ExportDefaultDeclarationKind::ClassDeclaration(class) => class.id.as_ref(),
			_ => None,
		};

		self.exports.push(ExportSpecifier {
			n:decl.exported.name().clone(),
			ln:ln.map(|id| id.name.clone()),
			s:decl.exported.span().start,
			e:decl.exported.span().end,
			ls:None,
			le:None,
			t:false,
		});
	}

	fn visit_export_all_declaration(&mut self, decl:&ExportAllDeclaration<'a>) {
		// export * as ns from 'foo'
		if let Some(exported) = &decl.exported {
			let n = exported.name().clone();

			let s = exported.span().start;

			let e = exported.span().end;

			self.exports.push(ExportSpecifier {
				n:n.clone(),
				ln:None,
				s,
				e,
				ls:None,
				le:None,
				t:decl.export_kind.is_type(),
			});

			self.imports.push(ImportSpecifier {
				n:Some(n),
				s,
				e,
				ss:decl.span.start,
				se:decl.span.end,
				d:ImportType::StaticImport,
				a:None,
				t:decl.export_kind.is_type(),
			});
		} else {
			// export * from 'foo'
			self.imports.push(ImportSpecifier {
				n:Some(decl.source.value.clone()),
				s:decl.source.span.start + 1, // +- 1 for removing string quotes
				e:decl.source.span.end - 1,
				ss:decl.span.start,
				se:decl.span.end,
				d:ImportType::ExportStar,
				a:None,
				t:decl.export_kind.is_type(),
			});
		}

		walk::walk_export_all_declaration(self, decl);
	}
}

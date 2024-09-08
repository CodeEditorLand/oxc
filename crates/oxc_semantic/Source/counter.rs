//! Visitor to count nodes, scopes, symbols and references in AST.
//! These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
//! `ScopeTree`, and `SymbolTable` to store info for all these items.

use std::cell::Cell;

use oxc_ast::{
    ast::{BindingIdentifier, IdentifierReference, TSEnumMemberName, TSModuleDeclarationName},
    visit::walk::{walk_ts_enum_member_name, walk_ts_module_declaration_name},
    AstKind, Visit,
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

#[allow(clippy::struct_field_names)]
#[derive(Default, Debug)]
pub(crate) struct Counter {
    pub nodes_count: usize,
    pub scopes_count: usize,
    pub symbols_count: usize,
    pub references_count: usize,
}

impl<'a> Visit<'a> for Counter {
    #[inline]
    fn enter_node(&mut self, _: AstKind<'a>) {
        self.nodes_count += 1;
    }

    #[inline]
    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.scopes_count += 1;
    }

    #[inline]
    fn visit_binding_identifier(&mut self, _: &BindingIdentifier<'a>) {
        self.nodes_count += 1;
        self.symbols_count += 1;
    }

    #[inline]
    fn visit_identifier_reference(&mut self, _: &IdentifierReference<'a>) {
        self.nodes_count += 1;
        self.references_count += 1;
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        if !it.is_expression() {
            self.symbols_count += 1;
        }
        walk_ts_enum_member_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_name(&mut self, it: &TSModuleDeclarationName<'a>) {
        self.symbols_count += 1;
        walk_ts_module_declaration_name(self, it);
    }
}

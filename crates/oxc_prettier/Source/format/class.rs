use std::ops::Add;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
	Format,
	Prettier,
	array,
	format::print::assignment::{AssignmentLikeNode, print_assignment},
	group,
	hardline,
	if_break,
	indent,
	ir::{Doc, JoinSeparator},
	join,
	line,
	softline,
	text,
};

pub fn print_class<'a>(p:&mut Prettier<'a>, class:&Class<'a>) -> Doc<'a> {
	let mut parts = Vec::new_in(p.allocator);
	let mut heritage_clauses_parts = Vec::new_in(p.allocator);
	let mut group_parts = Vec::new_in(p.allocator);

	// Keep old behaviour of extends in same line
	// If there is only on extends and there are not comments
	// ToDo: implement comment checks
	// @link <https://github.com/prettier/prettier/blob/aa3853b7765645b3f3d8a76e41cf6d70b93c01fd/src/language-js/print/class.js#L62>
	let group_mode = class.implements.as_ref().is_some_and(|v| !v.is_empty());

	if let Some(super_class) = &class.super_class {
		let mut extend_parts = Vec::new_in(p.allocator);

		extend_parts.push(text!("extends "));
		extend_parts.push(super_class.format(p));

		if let Some(super_type_parameters) = &class.super_type_parameters {
			extend_parts.push(super_type_parameters.format(p));
		}

		extend_parts.push(text!(" "));

		if group_mode {
			heritage_clauses_parts.push(softline!());
		}

		heritage_clauses_parts.push(array!(p, extend_parts));
	}

	heritage_clauses_parts.push(print_heritage_clauses_implements(p, class));

	for decorator in &class.decorators {
		parts.push(text!("@"));
		parts.push(decorator.expression.format(p));
		parts.extend(hardline!());
	}

	if class.declare {
		parts.push(text!("declare "));
	}

	if class.r#abstract {
		parts.push(text!("abstract "));
	}

	parts.push(text!("class "));

	if let Some(id) = &class.id {
		group_parts.push(id.format(p));
	}

	if let Some(params) = &class.type_parameters {
		group_parts.push(params.format(p));
	}

	if class.id.is_some() || class.type_parameters.is_some() {
		group_parts.push(text!(" "));
	}

	if group_mode {
		let printend_parts_group = if should_indent_only_heritage_clauses(class) {
			array!(p, [array!(p, group_parts), indent!(p, heritage_clauses_parts)])
		} else {
			indent!(p, [array!(p, group_parts), group!(p, heritage_clauses_parts)])
		};

		parts.push(printend_parts_group);

		if !class.body.body.is_empty() && has_multiple_heritage(class) {
			parts.extend(hardline!());
		}
	} else {
		parts.push(array!(p, [array!(p, group_parts), array!(p, heritage_clauses_parts)]));
	}

	parts.push(class.body.format(p));
	array!(p, parts)
}

pub fn print_class_body<'a>(p:&mut Prettier<'a>, class_body:&ClassBody<'a>) -> Doc<'a> {
	let mut parts_inner = Vec::new_in(p.allocator);

	for (i, node) in class_body.body.iter().enumerate() {
		parts_inner.push(node.format(p));

		if !p.options.semi
			&& node.is_property()
			&& should_print_semicolon_after_class_property(node, class_body.body.get(i + 1))
		{
			parts_inner.push(text!(";"));
		}

		if i < class_body.body.len() - 1 {
			parts_inner.extend(hardline!());

			if p.is_next_line_empty(node.span()) {
				parts_inner.extend(hardline!());
			}
		}
	}

	// TODO: if there are any dangling comments, print them

	let mut parts = Vec::new_in(p.allocator);
	parts.push(text!("{"));
	if !parts_inner.is_empty() {
		let indent = {
			let mut parts = Vec::new_in(p.allocator);
			parts.extend(hardline!());
			parts.push(array!(p, parts_inner));
			indent!(p, parts)
		};
		parts.push(array!(p, [indent]));
		parts.extend(hardline!());
	}

	parts.push(text!("}"));

	array!(p, parts)
}

#[derive(Debug)]
pub enum ClassMemberish<'a, 'b> {
	PropertyDefinition(&'b PropertyDefinition<'a>),
	AccessorProperty(&'b AccessorProperty<'a>),
}

impl<'a> ClassMemberish<'a, '_> {
	fn format_key(&self, p:&mut Prettier<'a>) -> Doc<'a> {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => {
				property_definition.key.format(p)
			},

			ClassMemberish::AccessorProperty(accessor_property) => accessor_property.key.format(p),
		}
	}

	fn decorators(&self) -> Option<&oxc_allocator::Vec<Decorator<'a>>> {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => {
				Some(&property_definition.decorators)
			},

			ClassMemberish::AccessorProperty(accessor_property) => None,
		}
	}

	fn is_static(&self) -> bool {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => property_definition.r#static,
			ClassMemberish::AccessorProperty(accessor_property) => accessor_property.r#static,
		}
	}

	fn is_override(&self) -> bool {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => {
				property_definition.r#override
			},

			ClassMemberish::AccessorProperty(accessor_property) => false,
		}
	}

	fn is_readonly(&self) -> bool {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => property_definition.readonly,
			ClassMemberish::AccessorProperty(_) => true,
		}
	}

	fn is_declare(&self) -> bool {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => property_definition.declare,
			ClassMemberish::AccessorProperty(_) => false,
		}
	}

	fn is_abstract(&self) -> bool {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => {
				property_definition.r#type == PropertyDefinitionType::TSAbstractPropertyDefinition
			},

			ClassMemberish::AccessorProperty(accessor_property) => {
				accessor_property.r#type == AccessorPropertyType::TSAbstractAccessorProperty
			},
		}
	}

	fn is_optional(&self) -> bool {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => property_definition.optional,
			ClassMemberish::AccessorProperty(_) => false,
		}
	}

	fn is_definite(&self) -> bool {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => property_definition.definite,
			ClassMemberish::AccessorProperty(accessor_property) => accessor_property.definite,
		}
	}

	fn right_expr(&self) -> Option<&Expression<'a>> {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => {
				property_definition.value.as_ref()
			},

			ClassMemberish::AccessorProperty(_) => None,
		}
	}

	fn format_accessibility(&self, p:&mut Prettier<'a>) -> Option<Doc<'a>> {
		match self {
			ClassMemberish::AccessorProperty(def) => def.accessibility.map(|v| text!(v.as_str())),
			ClassMemberish::PropertyDefinition(def) => def.accessibility.map(|v| text!(v.as_str())),
		}
	}

	fn format_type_annotation(&self, p:&mut Prettier<'a>) -> Option<Doc<'a>> {
		match self {
			ClassMemberish::PropertyDefinition(property_definition) => {
				property_definition
					.type_annotation
					.as_ref()
					.map(|v| v.type_annotation.format(p))
			},

			ClassMemberish::AccessorProperty(accessor_definition) => {
				accessor_definition
					.type_annotation
					.as_ref()
					.map(|v| v.type_annotation.format(p))
			},
		}
	}
}

pub fn print_class_property<'a>(p:&mut Prettier<'a>, node:&ClassMemberish<'a, '_>) -> Doc<'a> {
	let mut parts = Vec::new_in(p.allocator);

	if let Some(decarators) = node.decorators() {
		for decorator in decarators {
			parts.push(text!("@"));
			parts.push(decorator.expression.format(p));
			parts.extend(hardline!());
		}
	}

	if let Some(accessibility) = node.format_accessibility(p) {
		parts.push(accessibility);
		parts.push(text!(" "));
	}

	if node.is_declare() {
		parts.push(text!("declare "));
	}

	if node.is_static() {
		parts.push(text!("static "));
	}

	if node.is_abstract() {
		parts.push(text!("abstract "));
	}

	if node.is_override() {
		parts.push(text!("override "));
	}

	if node.is_readonly() {
		parts.push(text!("readonly "));
	}

	parts.push(node.format_key(p));

	if node.is_optional() {
		parts.push(text!("?"));
	} else if node.is_definite() {
		parts.push(text!("!"));
	}

	if let Some(type_annotation) = node.format_type_annotation(p) {
		parts.push(text!(": "));
		parts.push(type_annotation);
	}

	let right_expr = node.right_expr();

	let node = match node {
		ClassMemberish::PropertyDefinition(v) => AssignmentLikeNode::PropertyDefinition(v),
		ClassMemberish::AccessorProperty(v) => AssignmentLikeNode::AccessorProperty(v),
	};
	let mut result = print_assignment(p, node, array!(p, parts), text!(" ="), right_expr);

	if p.options.semi {
		let mut parts = Vec::new_in(p.allocator);
		parts.push(result);
		parts.push(text!(";"));
		result = array!(p, parts);
	}

	result
}

fn should_print_semicolon_after_class_property<'a>(
	node:&ClassElement<'a>,
	next_node:Option<&ClassElement<'a>>,
) -> bool {
	if !node.computed() {
		if let ClassElement::PropertyDefinition(property_definition) = node {
			if property_definition.value.is_none() && property_definition.type_annotation.is_none()
			{
				if let Some(key) = property_definition.key.static_name() {
					if key == "static" || key == "get" || key == "set" {
						return true;
					}
				}
			}
		}
	}

	let Some(next_node) = next_node else {
		return false;
	};

	if next_node.r#static() || next_node.accessibility().is_some() {
		return false;
	}

	if !next_node.computed() {
		if let Some(prop_key) = next_node.property_key() {
			if let Some(prop_key) = prop_key.static_name() {
				if prop_key == "in" || prop_key == "instanceof" {
					return true;
				}
			}
		}
	}

	match next_node {
		ClassElement::PropertyDefinition(property_definition) => property_definition.computed,
		ClassElement::StaticBlock(_) => false,
		ClassElement::AccessorProperty(_) | ClassElement::TSIndexSignature(_) => true,
		ClassElement::MethodDefinition(method_definition) => {
			let is_async = method_definition.value.r#async;

			if is_async
				|| method_definition.kind == MethodDefinitionKind::Get
				|| method_definition.kind == MethodDefinitionKind::Set
			{
				return false;
			}

			let is_generator = method_definition.value.generator;

			if method_definition.computed || is_generator {
				return true;
			}

			false
		},
	}
}

/// @link <https://github.com/prettier/prettier/blob/aa3853b7765645b3f3d8a76e41cf6d70b93c01fd/src/language-js/print/class.js#L148>
fn print_heritage_clauses_implements<'a>(p:&mut Prettier<'a>, class:&Class<'a>) -> Doc<'a> {
	let mut parts = Vec::new_in(p.allocator);

	if class.implements.is_none() {
		return array!(p, parts);
	}

	let implements = class.implements.as_ref().unwrap();

	if implements.len() == 0 {
		return array!(p, parts);
	}

	if should_indent_only_heritage_clauses(class) {
		parts.push(if_break!(p, line!()));
	} else if class.super_class.is_some() {
		parts.extend(hardline!());
	} else {
		parts.push(softline!());
	}

	parts.push(text!("implements "));

	let implements_docs = implements.iter().map(|v| v.format(p)).collect::<std::vec::Vec<_>>();

	parts.push(indent!(
		p,
		[group!(p, [softline!(), join!(p, JoinSeparator::CommaLine, implements_docs),])]
	));
	parts.push(text!(" "));

	group!(p, parts)
}

fn should_indent_only_heritage_clauses(class:&Class) -> bool {
	// Todo - Check for Comments
	// @link https://github.com/prettier/prettier/blob/aa3853b7765645b3f3d8a76e41cf6d70b93c01fd/src/language-js/print/class.js#L137
	class.type_parameters.is_some() && !has_multiple_heritage(class)
}

fn has_multiple_heritage(class:&Class) -> bool {
	let mut len = i32::from(class.super_class.is_some());

	if let Some(implements) = &class.implements {
		len = len.add(i32::try_from(implements.len()).unwrap());
	}

	len > 1
}

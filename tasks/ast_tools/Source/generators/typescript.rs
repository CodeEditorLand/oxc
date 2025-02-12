use convert_case::{Case, Casing};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};

use super::define_generator;
use crate::{
	Generator,
	TypeId,
	output::Output,
	schema::{
		EnumDef,
		FieldDef,
		GetIdent,
		Schema,
		StructDef,
		TypeDef,
		TypeName,
		serialize::{enum_variant_name, get_always_flatten_structs, get_type_tag},
	},
};

const CUSTOM_TYPESCRIPT:&str = include_str!("../../../../crates/oxc_ast/custom_types.d.ts");

pub struct TypescriptGenerator;

define_generator!(TypescriptGenerator);

impl Generator for TypescriptGenerator {
	fn generate(&mut self, schema:&Schema) -> Output {
		let mut code = String::new();

		let always_flatten_structs = get_always_flatten_structs(schema);

		for def in &schema.defs {
			if !def.generates_derive("ESTree") {
				continue;
			}

			let ts_type_def = match def {
				TypeDef::Struct(it) => Some(typescript_struct(it, &always_flatten_structs)),
				TypeDef::Enum(it) => typescript_enum(it),
			};

			let Some(ts_type_def) = ts_type_def else {
				continue;
			};

			code.push_str(&ts_type_def);

			code.push_str("\n\n");
		}

		code.push_str(CUSTOM_TYPESCRIPT);

		Output::Javascript { path:format!("{}/types.d.ts", crate::TYPESCRIPT_PACKAGE), code }
	}
}

// Untagged enums: `type Expression = BooleanLiteral | NullLiteral`
// Tagged enums: `type PropertyKind = 'init' | 'get' | 'set'`
fn typescript_enum(def:&EnumDef) -> Option<String> {
	if def.markers.estree.custom_ts_def {
		return None;
	}

	let is_untagged = def.all_variants().all(|var| var.fields.len() == 1);

	let union = if is_untagged {
		def.all_variants()
			.map(|var| type_to_string(var.fields[0].typ.name()))
			.join(" | ")
	} else {
		def.all_variants()
			.map(|var| format!("'{}'", enum_variant_name(var, def)))
			.join(" | ")
	};

	let ident = def.ident();

	Some(format!("export type {ident} = {union};"))
}

fn typescript_struct(def:&StructDef, always_flatten_structs:&FxHashSet<TypeId>) -> String {
	let ident = def.ident();

	let mut fields = String::new();

	let mut extends = vec![];

	if let Some(type_tag) = get_type_tag(def) {
		fields.push_str(&format!("\n\ttype: '{type_tag}';"));
	}

	let mut append_to:FxHashMap<String, &FieldDef> = FxHashMap::default();

	// Scan through to find all append_to fields
	for field in &def.fields {
		let Some(parent) = field.markers.derive_attributes.estree.append_to.as_ref() else {
			continue;
		};

		assert!(
			append_to.insert(parent.clone(), field).is_none(),
			"Duplicate append_to target (on {ident})"
		);
	}

	for field in &def.fields {
		if field.markers.derive_attributes.estree.skip
			|| field.markers.derive_attributes.estree.append_to.is_some()
		{
			continue;
		}

		let mut ty = match &field.markers.derive_attributes.estree.typescript_type {
			Some(ty) => ty.clone(),
			None => type_to_string(field.typ.name()),
		};

		let always_flatten = match field.typ.type_id() {
			Some(id) => always_flatten_structs.contains(&id),
			None => false,
		};

		if always_flatten || field.markers.derive_attributes.estree.flatten {
			extends.push(ty);

			continue;
		}

		let ident = field.ident().unwrap();

		if let Some(append_after) = append_to.get(&ident.to_string()) {
			let ts_type = &append_after.markers.derive_attributes.estree.typescript_type;

			let after_type = if let Some(ty) = ts_type {
				ty.clone()
			} else {
				let typ = append_after.typ.name();

				if let TypeName::Opt(inner) = typ {
					type_to_string(inner)
				} else {
					panic!(
						"expected field labeled with append_to to be Option<...>, but found {typ}"
					);
				}
			};

			if let Some(inner) = ty.strip_prefix("Array<") {
				ty = format!("Array<{after_type} | {inner}");
			} else {
				panic!("expected append_to target to be a Vec, but found {ty}");
			}
		}

		let name = match &field.markers.derive_attributes.estree.rename {
			Some(rename) => rename.to_string(),
			None => field.name.clone().unwrap().to_case(Case::Camel),
		};

		fields.push_str(&format!("\n\t{name}: {ty};"));
	}

	let extends_union = extends.iter().any(|it| it.contains('|'));

	let body = if let Some(extra_ts) = def.markers.estree.as_ref().and_then(|e| e.add_ts.as_ref()) {
		format!("{{{fields}\n\t{extra_ts}\n}}")
	} else {
		format!("{{{fields}\n}}")
	};

	if extends_union {
		let extends = if extends.is_empty() {
			String::new()
		} else {
			format!(" & {}", extends.join(" & "))
		};

		format!("export type {ident} = ({body}){extends};")
	} else {
		let extends = if extends.is_empty() {
			String::new()
		} else {
			format!(" extends {}", extends.join(", "))
		};

		format!("export interface {ident}{extends} {body}")
	}
}

fn type_to_string(ty:&TypeName) -> String {
	match ty {
		TypeName::Ident(ident) => {
			match ident.as_str() {
				"f64" | "f32" | "usize" | "u64" | "u32" | "u16" | "u8" | "i64" | "i32" | "i16"
				| "i8" => "number",
				"bool" => "boolean",
				"str" | "String" | "Atom" | "CompactStr" => "string",
				ty => ty,
			}
			.to_string()
		},
		TypeName::Vec(type_name) => format!("Array<{}>", type_to_string(type_name)),
		TypeName::Box(type_name) | TypeName::Ref(type_name) | TypeName::Complex(type_name) => {
			type_to_string(type_name)
		},

		TypeName::Opt(type_name) => format!("{} | null", type_to_string(type_name)),
	}
}

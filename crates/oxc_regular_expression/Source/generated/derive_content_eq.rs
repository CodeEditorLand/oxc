// Auto-generated code, DO NOT EDIT DIRECTLY!
// To edit this generated file you have to edit
// `tasks/ast_tools/src/derives/content_eq.rs`

#![allow(clippy::match_like_matches_macro)]

use oxc_span::cmp::ContentEq;

use crate::ast::*;

impl ContentEq for Pattern<'_> {
	fn content_eq(&self, other:&Self) -> bool { ContentEq::content_eq(&self.body, &other.body) }
}

impl ContentEq for Disjunction<'_> {
	fn content_eq(&self, other:&Self) -> bool { ContentEq::content_eq(&self.body, &other.body) }
}

impl ContentEq for Alternative<'_> {
	fn content_eq(&self, other:&Self) -> bool { ContentEq::content_eq(&self.body, &other.body) }
}

impl ContentEq for Term<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		match self {
			Self::BoundaryAssertion(it) => {
				match other {
					Self::BoundaryAssertion(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::LookAroundAssertion(it) => {
				match other {
					Self::LookAroundAssertion(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::Quantifier(it) => {
				match other {
					Self::Quantifier(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::Character(it) => {
				match other {
					Self::Character(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::Dot(it) => {
				match other {
					Self::Dot(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::CharacterClassEscape(it) => {
				match other {
					Self::CharacterClassEscape(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::UnicodePropertyEscape(it) => {
				match other {
					Self::UnicodePropertyEscape(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::CharacterClass(it) => {
				match other {
					Self::CharacterClass(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::CapturingGroup(it) => {
				match other {
					Self::CapturingGroup(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::IgnoreGroup(it) => {
				match other {
					Self::IgnoreGroup(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::IndexedReference(it) => {
				match other {
					Self::IndexedReference(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::NamedReference(it) => {
				match other {
					Self::NamedReference(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
		}
	}
}

impl ContentEq for BoundaryAssertion {
	fn content_eq(&self, other:&Self) -> bool { ContentEq::content_eq(&self.kind, &other.kind) }
}

impl ContentEq for BoundaryAssertionKind {
	fn content_eq(&self, other:&Self) -> bool { self == other }
}

impl ContentEq for LookAroundAssertion<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.kind, &other.kind)
			&& ContentEq::content_eq(&self.body, &other.body)
	}
}

impl ContentEq for LookAroundAssertionKind {
	fn content_eq(&self, other:&Self) -> bool { self == other }
}

impl ContentEq for Quantifier<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.min, &other.min)
			&& ContentEq::content_eq(&self.max, &other.max)
			&& ContentEq::content_eq(&self.greedy, &other.greedy)
			&& ContentEq::content_eq(&self.body, &other.body)
	}
}

impl ContentEq for Character {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.kind, &other.kind)
			&& ContentEq::content_eq(&self.value, &other.value)
	}
}

impl ContentEq for CharacterKind {
	fn content_eq(&self, other:&Self) -> bool { self == other }
}

impl ContentEq for CharacterClassEscape {
	fn content_eq(&self, other:&Self) -> bool { ContentEq::content_eq(&self.kind, &other.kind) }
}

impl ContentEq for CharacterClassEscapeKind {
	fn content_eq(&self, other:&Self) -> bool { self == other }
}

impl ContentEq for UnicodePropertyEscape<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.negative, &other.negative)
			&& ContentEq::content_eq(&self.strings, &other.strings)
			&& ContentEq::content_eq(&self.name, &other.name)
			&& ContentEq::content_eq(&self.value, &other.value)
	}
}

impl ContentEq for Dot {
	fn content_eq(&self, _:&Self) -> bool { true }
}

impl ContentEq for CharacterClass<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.negative, &other.negative)
			&& ContentEq::content_eq(&self.strings, &other.strings)
			&& ContentEq::content_eq(&self.kind, &other.kind)
			&& ContentEq::content_eq(&self.body, &other.body)
	}
}

impl ContentEq for CharacterClassContentsKind {
	fn content_eq(&self, other:&Self) -> bool { self == other }
}

impl ContentEq for CharacterClassContents<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		match self {
			Self::CharacterClassRange(it) => {
				match other {
					Self::CharacterClassRange(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::CharacterClassEscape(it) => {
				match other {
					Self::CharacterClassEscape(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::UnicodePropertyEscape(it) => {
				match other {
					Self::UnicodePropertyEscape(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::Character(it) => {
				match other {
					Self::Character(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::NestedCharacterClass(it) => {
				match other {
					Self::NestedCharacterClass(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
			Self::ClassStringDisjunction(it) => {
				match other {
					Self::ClassStringDisjunction(other) if ContentEq::content_eq(it, other) => true,
					_ => false,
				}
			},
		}
	}
}

impl ContentEq for CharacterClassRange {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.min, &other.min) && ContentEq::content_eq(&self.max, &other.max)
	}
}

impl ContentEq for ClassStringDisjunction<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.strings, &other.strings)
			&& ContentEq::content_eq(&self.body, &other.body)
	}
}

impl ContentEq for ClassString<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.strings, &other.strings)
			&& ContentEq::content_eq(&self.body, &other.body)
	}
}

impl ContentEq for CapturingGroup<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.name, &other.name)
			&& ContentEq::content_eq(&self.body, &other.body)
	}
}

impl ContentEq for IgnoreGroup<'_> {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.modifiers, &other.modifiers)
			&& ContentEq::content_eq(&self.body, &other.body)
	}
}

impl ContentEq for Modifiers {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.enabling, &other.enabling)
			&& ContentEq::content_eq(&self.disabling, &other.disabling)
	}
}

impl ContentEq for Modifier {
	fn content_eq(&self, other:&Self) -> bool {
		ContentEq::content_eq(&self.ignore_case, &other.ignore_case)
			&& ContentEq::content_eq(&self.multiline, &other.multiline)
			&& ContentEq::content_eq(&self.sticky, &other.sticky)
	}
}

impl ContentEq for IndexedReference {
	fn content_eq(&self, other:&Self) -> bool { ContentEq::content_eq(&self.index, &other.index) }
}

impl ContentEq for NamedReference<'_> {
	fn content_eq(&self, other:&Self) -> bool { ContentEq::content_eq(&self.name, &other.name) }
}

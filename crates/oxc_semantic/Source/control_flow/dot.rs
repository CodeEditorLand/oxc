use oxc_ast::{
	AstKind,
	ast::{BreakStatement, ContinueStatement},
};
use oxc_syntax::node::AstNodeId;
use petgraph::{
	dot::{Config, Dot},
	visit::EdgeRef,
};

use super::IterationInstructionKind;
use crate::{
	AstNode,
	AstNodes,
	BasicBlock,
	ControlFlowGraph,
	EdgeType,
	Instruction,
	InstructionKind,
	LabeledInstruction,
	ReturnInstructionKind,
};

pub trait DisplayDot {
	fn display_dot(&self) -> String;
}

pub trait DebugDot {
	fn debug_dot(&self, ctx:DebugDotContext) -> String;
}

#[derive(Clone, Copy)]
pub struct DebugDotContext<'a, 'b>(&'b AstNodes<'a>);

impl<'a, 'b> DebugDotContext<'a, 'b> {
	fn debug_ast_kind(self, id:AstNodeId) -> String { self.0.kind(id).debug_name().into_owned() }

	fn try_eval_literal(self, id:AstNodeId) -> Option<String> {
		match self.0.kind(id) {
			AstKind::NumericLiteral(lit) => Some(lit.value.to_string()),
			AstKind::BooleanLiteral(lit) => Some(lit.value.to_string()),
			AstKind::StringLiteral(lit) => Some(lit.value.to_string()),
			AstKind::BigintLiteral(lit) => Some(lit.raw.to_string()),
			AstKind::NullLiteral(_) => Some("null".to_string()),
			_ => None,
		}
	}
}

impl<'a, 'b> From<&'b AstNodes<'a>> for DebugDotContext<'a, 'b> {
	fn from(value:&'b AstNodes<'a>) -> Self { Self(value) }
}

impl DisplayDot for ControlFlowGraph {
	fn display_dot(&self) -> String {
		format!(
			"{:?}",
			Dot::with_attr_getters(
				&self.graph,
				&[Config::EdgeNoLabel, Config::NodeNoLabel],
				&|_graph, edge| {
					let weight = edge.weight();

					let label = format!("label = \"{weight:?}\" ");

					if matches!(weight, EdgeType::Unreachable)
						|| self.basic_block(edge.source()).unreachable
					{
						format!("{label}, style = \"dotted\" ")
					} else {
						label
					}
				},
				&|_graph, node| {
					format!("label = {:?} ", self.basic_blocks[*node.1].display_dot().trim())
				},
			)
		)
	}
}

impl DisplayDot for BasicBlock {
	fn display_dot(&self) -> String {
		self.instructions().iter().fold(String::new(), |mut acc, it| {
			acc.push_str(it.display_dot().as_str());

			acc.push('\n');

			acc
		})
	}
}

impl DisplayDot for Instruction {
	fn display_dot(&self) -> String {
		match self.kind {
			InstructionKind::Statement => "statement",
			InstructionKind::Unreachable => "unreachable",
			InstructionKind::Throw => "throw",
			InstructionKind::Condition => "condition",
			InstructionKind::Iteration(IterationInstructionKind::Of) => "iteration <of>",
			InstructionKind::Iteration(IterationInstructionKind::In) => "iteration <in>",
			InstructionKind::Break(LabeledInstruction::Labeled) => "break <label>",
			InstructionKind::Break(LabeledInstruction::Unlabeled) => "break",
			InstructionKind::Continue(LabeledInstruction::Labeled) => "continue <label>",
			InstructionKind::Continue(LabeledInstruction::Unlabeled) => "continue",
			InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined) => {
				"return <implicit undefined>"
			},
			InstructionKind::Return(ReturnInstructionKind::NotImplicitUndefined) => {
				"return <value>"
			},
		}
		.to_string()
	}
}

impl DebugDot for ControlFlowGraph {
	fn debug_dot(&self, ctx:DebugDotContext) -> String {
		format!(
			"{:?}",
			Dot::with_attr_getters(
				&self.graph,
				&[Config::EdgeNoLabel, Config::NodeNoLabel],
				&|_graph, edge| {
					let weight = edge.weight();

					let label = format!("label = \"{weight:?}\" ");

					if matches!(weight, EdgeType::Unreachable)
						|| self.basic_block(edge.source()).unreachable
					{
						format!("{label}, style = \"dotted\" ")
					} else {
						label
					}
				},
				&|_graph, node| {
					format!("label = {:?} ", self.basic_blocks[*node.1].debug_dot(ctx).trim())
				},
			)
		)
	}
}

impl DebugDot for BasicBlock {
	fn debug_dot(&self, ctx:DebugDotContext) -> String {
		self.instructions().iter().fold(String::new(), |mut acc, it| {
			acc.push_str(it.debug_dot(ctx).as_str());

			acc.push('\n');

			acc
		})
	}
}

impl DebugDot for Instruction {
	fn debug_dot(&self, ctx:DebugDotContext) -> String {
		match self.kind {
			InstructionKind::Statement => {
				self.node_id.map_or("None".to_string(), |id| ctx.debug_ast_kind(id))
			},
			InstructionKind::Unreachable => "unreachable".to_string(),
			InstructionKind::Throw => "throw".to_string(),
			InstructionKind::Condition => {
				self.node_id.map_or("None".to_string(), |id| {
					format!(
						"Condition({})",
						ctx.try_eval_literal(id).unwrap_or_else(|| ctx.debug_ast_kind(id))
					)
				})
			},
			InstructionKind::Iteration(ref kind) => {
				format!(
					"Iteration({} {} {})",
					self.node_id.map_or("None".to_string(), |id| { ctx.debug_ast_kind(id) }),
					if matches!(kind, IterationInstructionKind::Of) { "of" } else { "in" },
					// TODO: at this point we can't evaluate this note. needs
					// access to the graph information.
					"expr"
				)
			},
			InstructionKind::Break(LabeledInstruction::Labeled) => {
				let Some(AstKind::BreakStatement(BreakStatement { label: Some(label), .. })) =
					self.node_id.map(|id| ctx.0.get_node(id)).map(AstNode::kind)
				else {
					unreachable!(
						"Expected a label node to be associated with an labeled break \
						 instruction. {:?}",
						ctx.0.kind(self.node_id.unwrap())
					)
				};

				format!("break <{}>", label.name)
			},
			InstructionKind::Break(LabeledInstruction::Unlabeled) => "break".to_string(),
			InstructionKind::Continue(LabeledInstruction::Labeled) => {
				let Some(AstKind::ContinueStatement(ContinueStatement {
					label: Some(label), ..
				})) = self.node_id.map(|id| ctx.0.get_node(id)).map(AstNode::kind)
				else {
					unreachable!(
						"Expected a label node to be associated with an labeled continue \
						 instruction. {:?}",
						ctx.0.kind(self.node_id.unwrap())
					)
				};

				format!("continue <{}>", label.name)
			},
			InstructionKind::Continue(LabeledInstruction::Unlabeled) => "continue".to_string(),
			InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined) => {
				"return <implicit undefined>".to_string()
			},
			InstructionKind::Return(ReturnInstructionKind::NotImplicitUndefined) => {
				"return <value>".to_string()
			},
		}
	}
}

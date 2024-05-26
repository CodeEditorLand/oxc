const typeFilter = ["JsonText", "Object", "Property", "Array"];
/**
 * @param {import('@lezer/common').SyntaxNode} node
 * @returns {import('@lezer/common').SyntaxNode}
 *
 * */
export function findMostInnerNodeForPosition(node, offset, source) {
	if (!typeFilter.includes(node.name)) {
		return;
	}
	let targetNode;
	if (node.name === "Object") {
		const span = getSpanOfNode(node, source);
		if (Object.keys(span).length > 0) {
			const { start, end } = span;
			if (start <= offset && end >= offset) {
				targetNode = node;
			} else {
				return targetNode;
			}
		}
	}
	let curChild = node.firstChild;
	while (curChild) {
		const node = findMostInnerNodeForPosition(curChild, offset, source);
		if (node?.from) {
			targetNode = node;
		}
		curChild = curChild.nextSibling;
	}
	return targetNode;
}

/**
 * @param {import('@lezer/common').SyntaxNode} node
 * @param {string} source
 * */
function getSpanOfNode(node, source) {
	const span = {};
	let child = node.firstChild;
	while (child) {
		if (
			child.name === "Property" &&
			child.firstChild.name === "PropertyName"
		) {
			const { from, to } = child.firstChild;
			const name = source.slice(from + 1, to - 1);
			if (["start", "end"].includes(name)) {
				const value = child.firstChild.nextSibling;
				if (value) {
					span[name] = +source.slice(value.from, value.to);
				}
			}
		}
		child = child.nextSibling;
	}
	return span;
}

use tree_sitter::{Node, Point, TreeCursor};

// TODO: Handle wildcards
pub fn collect_imports(mut cursor: TreeCursor) -> Vec<Node> {
    let mut imports = Vec::new();

    loop {
        let node = cursor.node();

        if node.kind() == "import_declaration" && node.child_count() > 1 {
            imports.push(node.child(1).unwrap());
        };

        if !cursor.goto_next_sibling() && !cursor.goto_first_child() {
            break;
        }
    }

    imports
}

pub fn find_node_by_point(mut cursor: TreeCursor, point: Point) -> Node {
    while cursor.goto_first_child_for_point(point).is_some() {}

    cursor.node()
}

//! Module with logic for node collapsing.
//!
//! See the [`collapse`] function for details.

use crate::prelude::*;

use crate::double_representation::definition::{DefinitionInfo, DefinitionName};
use crate::double_representation::definition;
use crate::double_representation::identifier::Identifier;
use crate::double_representation::node;
use crate::double_representation::graph::GraphInfo;

use parser::Parser;
use crate::double_representation::connection::{Connection, Endpoint};
use crate::double_representation::node::NodeInfo;
use wasm_bindgen::__rt::std::collections::BTreeSet;


#[derive(Clone,Debug)]
pub struct Collapsed {
    pub updated_definition:DefinitionInfo,
    pub new_method:definition::ToAdd,
}

#[derive(Clone,Debug,Default)]
struct ClassifiedConnections {
    inputs  : Vec<Connection>,
    outputs : Option<Connection>,
}

impl ClassifiedConnections {
    fn new(selected_nodes:&HashSet<node::Id>, all_connections:Vec<Connection>) -> ClassifiedConnections {
        let mut inputs  = Vec::new();
        let mut outputs = Vec::new();
        for connection in all_connections {
            let starts_inside = selected_nodes.contains(&connection.source.node);
            let ends_inside   = selected_nodes.contains(&connection.destination.node);
            if !starts_inside && ends_inside {
                inputs.push(connection)
            } else if starts_inside && !ends_inside {
                outputs.push(connection)
            }
        }

        let outputs = if outputs.len() > 1 {
            panic!("Too many outputs"); // TODO TODO TODO
        } else {
            outputs.into_iter().next()
        };
        //let outputs = outputs.into_iter().next();

        Self {inputs,outputs}
    }
}

pub fn collapse
(graph:&GraphInfo, selected_nodes:impl IntoIterator<Item=node::Id>, name:DefinitionName, parser:&Parser)
-> FallibleResult<Collapsed> {
    let endpoint_to_node = |endpoint:&Endpoint| {
        graph.find_node(endpoint.node).unwrap() // TODO
    };
    let endpoint_to_identifier = |endpoint:&Endpoint| {
        let node = endpoint_to_node(endpoint);
        Identifier::new(node.ast().get_traversing(&endpoint.crumbs).unwrap().clone_ref()).unwrap()
    };

    // We need to have both vector and set -- one keeps the nodes (lines) in order,
    // while the other one allows for a fast lookup.
    let selected_nodes = selected_nodes.into_iter().collect_vec();
    let last_selected_node = selected_nodes.iter().last().unwrap(); // TODO
    let selected_nodes_set = selected_nodes.iter().copied().collect::<HashSet<_>>();
    let connections    = graph.connections();
    let connections    = ClassifiedConnections::new(&selected_nodes_set,connections);

    let inputs = connections.inputs.iter().map(|connection| {
        // Here it doesn't really matter what endpoint we take (src or dst), as they both
        // should be occurrence of the same variable.
        endpoint_to_identifier(&connection.source)
    }).collect::<BTreeSet<_>>();

    let output_node = connections.outputs.as_ref().map(|output_connection| {
        endpoint_to_node(&output_connection.source)
    });
    let output_node_id = output_node.as_ref().map(NodeInfo::id);
    let node_to_replace = output_node_id.unwrap_or(*last_selected_node);


    let return_line : Option<Ast> = connections.outputs.map(|c| endpoint_to_identifier(&c.source).deref().clone());
    let mut selected_nodes_iter = selected_nodes.iter().map(|node| graph.find_node(*node).unwrap().ast().clone());

    let body_head                = selected_nodes_iter.next().unwrap();
    let body_tail                = selected_nodes_iter.chain(return_line).map(Some).collect();
    let explicit_parameter_names = inputs.iter().map(|input| input.name().to_owned()).collect();
    let to_add = definition::ToAdd {name,explicit_parameter_names,body_head,body_tail};

    let mut updated_def = graph.source.clone();
    let mut lines = updated_def.block_lines()?;
    lines.drain_filter(|line| {
        // There are 3 kind of lines:
        // 1) Lines that are left intact -- not belonging to selected nodes;
        // 2) Lines that are extracted and removed -- all selected nodes, except:
        // 3) Line that introduces output of the extracted function (if present at all) -> its
        //    expression shall be replaced with a call to the extracted function.
        //    If there is no usage of the extracted function output, its invocation should be placed
        //    in place of the last extracted line.
        let mut node_info = match line.elem.as_ref().and_then(NodeInfo::from_line_ast) {
            Some(node_info) => node_info,
            _               => return false, // We leave lines without nodes (blank lines) intact.
        };
        let node_id     = node_info.id();
        let is_selected = selected_nodes_set.contains(&node_id);
        if !is_selected {
            println!("Leaving {} intact.", node_info.ast());
            false
        } else if node_id == node_to_replace {
            let old_ast = node_info.ast().clone_ref();
            let base = to_add.name.ast(&parser).unwrap();
            let args = to_add.explicit_parameter_names.iter().map(Ast::var);
            let invocation = ast::prefix::Chain::new(base,args);
            node_info.set_expression(invocation.into_ast());
            if output_node_id.is_none() {
                node_info.clear_pattern()
            }

            let new_ast = node_info.ast().clone_ref();
            println!("Rewriting {} into a call {}.", old_ast, new_ast);
            line.elem = Some(new_ast); // TODO TODO TODO
            false
        } else {
            println!("Extracting {} out.", node_info.ast());
            true
        }
    });
    updated_def.set_block_lines(lines)?;

    Ok(Collapsed {
        new_method : to_add,
        updated_definition : updated_def,
    })
}



// ============
// === Test ===
// ============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::double_representation::graph;
    use crate::double_representation::module;
    use crate::double_representation::node::NodeInfo;

    struct Case {
        refactored_name : DefinitionName,
        introduced_name : DefinitionName,
        initial_method_code : String,
        extracted_lines     : Range<usize>,
        expected_generated  : String,
        expected_refactored : String
    }

    impl Case {
        fn from_lines(initial_method_lines:&[&str], extracted_lines:Range<usize>, expected_generated_lines:&[&str], expected_refactored_lines:&[&str]) -> Case {
            use crate::test::mock::def_from_lines;;
            let refactored_name = DefinitionName::new_plain("main");
            let introduced_name = DefinitionName::new_plain("func1");
            let initial_method_code = def_from_lines(&refactored_name,initial_method_lines);
            let expected_generated  = def_from_lines(&introduced_name,expected_generated_lines);
            let expected_refactored = def_from_lines(&refactored_name,expected_refactored_lines);
            Case {refactored_name,introduced_name,initial_method_code,extracted_lines,
                expected_generated,expected_refactored}
        }

        fn run(&self, parser:&Parser) {
            let ast   = parser.parse_module(&self.initial_method_code, default()).unwrap();
            let main  = module::locate_child(&ast,&self.refactored_name).unwrap();
            let graph = graph::GraphInfo::from_definition(main.item.clone());
            let nodes = graph.nodes();

            let selected_nodes = nodes[self.extracted_lines.clone()].iter().map(NodeInfo::id);

            let collapsed = collapse(&graph,selected_nodes,self.introduced_name.clone(),parser).unwrap();

            let new_method = collapsed.new_method.ast(0,parser).unwrap();
            let placement  = module::Placement::Before(self.refactored_name.clone());
            let new_main = &collapsed.updated_definition.ast;
            println!("Generated method:\n{}",new_method);
            println!("Updated method:\n{}",new_main);
            let mut module = module::Info{ast};
            module.ast = module.ast.set(&main.crumb().into(),new_main.ast().clone()).unwrap();
            module.add_method(collapsed.new_method,placement,parser).unwrap();
            println!("Module after refactoring:\n{}",&module.ast);

            assert_eq!(new_method.repr(),self.expected_generated);
            assert_eq!(new_main.repr(),self.expected_refactored);
        }
    }

    #[test]
    fn test_collapse() {
        let parser              = Parser::new_or_panic();
        let introduced_name = DefinitionName::new_plain("custom_new");
        let refactored_name = DefinitionName::new_plain("custom_old");
        let initial_method_code = r"custom_old =
    a = 1
    b = 2
    c = A + B
    d = a + b
    c + 7".to_owned();
        let extracted_lines    = 1..4;
        let expected_generated = r"custom_new a =
    b = 2
    c = A + B
    d = a + b
    c".to_owned();
        let expected_refactored = r"custom_old =
    a = 1
    c = custom_new a
    c + 7".to_owned();

        let mut case = Case {refactored_name,introduced_name,initial_method_code,
            extracted_lines,expected_generated,expected_refactored};
        case.run(&parser);


        // ========================================================================================
        // Check that refactoring a single assignment line:
        // 1) Maintains the assignment and the introduced name for the value in the extracted
        //    method;
        // 2) That invocation appears in the extracted node's place but has no assignment.

        case.extracted_lines = 3..4;
        case.expected_generated = r"custom_new a b =
    d = a + b".to_owned();
        case.expected_refactored = r"custom_old =
    a = 1
    b = 2
    c = A + B
    custom_new a b
    c + 7".to_owned();
        case.run(&parser);

        // ========================================================================================
        // Check that refactoring a single non-assignment line:
        // 1) Maintains the assignment and the introduced name for the value in the extracted
        //    method;
        // 2) That invocation appears in the extracted node's place but has no assignment.
    }
}
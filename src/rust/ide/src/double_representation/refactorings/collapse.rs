use crate::prelude::*;

use crate::double_representation::Identifier;
use crate::double_representation::definition::{DefinitionInfo, DefinitionName};
use crate::double_representation::definition;
use crate::double_representation::node;
use crate::double_representation::graph::GraphInfo;

use parser::Parser;
use crate::double_representation::connection::{Connection, Endpoint};
use crate::double_representation::node::NodeInfo;



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

// struct Collapser {
//     selected_nodes_vec:Vec<node::NodeInfo>,
//     selected_nodes_set:HashSet<node::Id>,
// }

pub fn collapse
(graph:&GraphInfo, selected_nodes:impl IntoIterator<Item=node::Id>, parser:&Parser)
-> FallibleResult<Collapsed> {
    let endpoint_to_node = |endpoint:&Endpoint| {
        let Endpoint {node,crumbs} = &endpoint;
        graph.find_node(*node).unwrap() // TODO
    };
    let endpoint_to_identifier = |endpoint:&Endpoint| {
        let node = endpoint_to_node(endpoint);
        Identifier::new(node.ast().get_traversing(&endpoint.crumbs).unwrap().clone_ref()).unwrap()
    };

    // We need to have both vector and set -- one keeps the nodes (lines) in order,
    // while the other one allows for a fast lookup.
    let selected_nodes = selected_nodes.into_iter().collect_vec();
    let selected_nodes_set = selected_nodes.iter().copied().collect::<HashSet<_>>();
    let connections    = graph.connections();
    let connections    = ClassifiedConnections::new(&selected_nodes_set,connections);

    let inputs = connections.inputs.iter().map(|connection| {
        // Here it doesn't really matter what endpoint we take (src or dst), as they both
        // should be occurrence of the same variable.
        endpoint_to_identifier(&connection.source)
    }).collect::<HashSet<_>>();

    let output_node = connections.outputs.as_ref().map(|output_connection| {
        endpoint_to_node(&output_connection.source)
    });
    let output_node_id = output_node.as_ref().map(NodeInfo::id);


    let return_line : Option<Ast> = connections.outputs.map(|c| endpoint_to_identifier(&c.source).deref().clone());
    let mut selected_nodes_iter = selected_nodes.iter().map(|node| graph.find_node(*node).unwrap().ast().clone());

    let body_head                = selected_nodes_iter.next().unwrap();
    let body_tail                = selected_nodes_iter.chain(return_line).map(Some).collect();
    let name                     = DefinitionName::new_plain("func1");
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
        let mut node_info = match line.elem.as_ref().and_then(NodeInfo::from_line_ast) {
            Some(node_info) => node_info,
            _               => return false, // We leave lines without nodes (blank lines) intact.
        };
        let node_id     = node_info.id();
        let is_selected = selected_nodes_set.contains(&node_id);
        let is_output   = output_node_id.contains(&node_id);
        if !is_selected {
            println!("Leaving {} intact.", node_info.ast());
            false
        } else if is_output {
            let old_ast = node_info.ast().clone_ref();
            let base = to_add.name.ast(&parser).unwrap();
            let args = to_add.explicit_parameter_names.iter().map(Ast::var);
            let invocation = ast::prefix::Chain::new(base,args);
            node_info.set_expression(invocation.into_ast());
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

    use double_representation::module;
    use double_representation::definition;
    use double_representation::graph;
    use crate::double_representation::node::NodeInfo;
    use crate::double_representation::module::Placement;


    #[test]
    fn collapse() {
        let code = r"main =
    a = 1
    b = 2
    c = A + B
    d = a + b
    c + 7";

        let parser = Parser::new_or_panic();

        let module = parser.parse_module(code,default()).unwrap();
        let mut module = module::Info {ast:module};

        let main_name = definition::DefinitionName::new_plain("main");
        let main = module::locate_child(&module.ast,&main_name).unwrap();
        let graph = graph::GraphInfo::from_definition(main.item.clone());
        let nodes = graph.nodes();

        let selected_nodes = nodes[1..4].iter().map(NodeInfo::id);

        let collapsed = super::collapse(&graph,selected_nodes,&parser).unwrap();

        let new_method = collapsed.new_method.ast(0,&parser).unwrap();
        let new_main = &collapsed.updated_definition.ast;
        println!("Generated method:\n{}",new_method);
        println!("Updated method:\n{}",new_main);
        module.ast = module.ast.set(&main.crumb().into(),new_main.ast().clone()).unwrap();
        module.add_method(collapsed.new_method,module::Placement::Before(main_name),&parser).unwrap();
        println!("Module after refactoring:\n{}",&module.ast);


        //dbg!();

        //dbg!(&main);





    }
}
//! The Subsequence Graph.
//!
//!
// TODO[ao] detailed explanation of graph structure.
use crate::prelude::*;

use std::collections::BTreeSet;
use std::cmp::min;



// =============
// === Graph ===
// =============

/// A graph vertex.
///
/// The vertices are identified by two indexes of chars: in word, and in query. See module docs for
/// details.
///
/// The fields' order is significant, because it affect how those are ordered in graph's vertices
/// list.
#[derive(Copy,Clone,Debug,Eq,Hash,Ord,PartialEq,PartialOrd)]
pub struct Vertex {
    pub query_char_index : usize,
    pub word_char_index  : usize,
}

/// A graph edge.
///
/// The fields' order is significant, because it affect how those are ordered in graph's edges
/// list.
#[derive(Copy,Clone,Debug,Eq,Hash,Ord,PartialEq,PartialOrd)]
pub struct Edge {
    pub from : Vertex,
    pub to   : Vertex,
}

/// The Subsequence Graph.
///
/// See module docs for detailed description of Subsequence Graph. We keep vertices and edges
/// ordered, because the scoring algorithm requires this ordering to be effective.
#[allow(missing_docs)]
#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct Graph {
    pub vertices : BTreeSet<Vertex>,
    pub edges    : BTreeSet<Edge>,
}

impl Graph {
    /// Generate graph based on `word` and `query`.
    pub fn new(word:impl Str, query:impl Str) -> Self {
        let vertices = Self::create_vertices(word.as_ref(),query.as_ref());
        let edges    = Self::create_edges(&vertices);
        Graph{vertices,edges}
    }

    fn create_vertices(word:&str, query:&str) -> BTreeSet<Vertex> {
        let mut result                    = BTreeSet::default();
        let mut first_reachable_word_char = 0;
        for (i,query_ch) in query.chars().enumerate() {
            let to_skip = first_reachable_word_char;
            first_reachable_word_char = word.len();
            for (j,word_ch) in word.chars().enumerate().skip(to_skip) {
                if query_ch.eq_ignore_ascii_case(&word_ch) {
                    result.insert(Vertex {query_char_index:i, word_char_index:j});
                    first_reachable_word_char = min(first_reachable_word_char,j);
                }
            }
        }
        result
    }

    fn create_edges(vertices:&BTreeSet<Vertex>) -> BTreeSet<Edge> {
        let mut result = BTreeSet::default();
        for from in vertices {
            let first_possible_to = Vertex{
                query_char_index : from.query_char_index + 1,
                word_char_index  : from.word_char_index  + 1,
            };
            let first_impossible_to = Vertex{
                query_char_index : from.query_char_index + 2,
                word_char_index  : 0,
            };
            for to in vertices.range(first_possible_to..first_impossible_to) {
                result.insert(Edge{from:*from, to:*to});
            }
        }
        result
    }

    pub fn vertices_with_query_char_index(&self, index:usize) -> impl Iterator<Item=&Vertex> {
        let start = Vertex{query_char_index:index    , word_char_index:0};
        let end   = Vertex{query_char_index:index + 1, word_char_index:0};
        self.vertices.range(start..end)
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generating_graph() {
        struct Case {
            word     : &'static str,
            query    : &'static str,
            vertices : Vec<(usize,usize)>,
            edges    : Vec<((usize,usize),(usize,usize))>,
        }

        impl Case {
            fn run(self) {
                let graph = Graph::new(self.word,self.query);
                let expected_vertices = self.vertices.into_iter().map(Self::convert_vertex);
                let expected_edges    = self.edges.into_iter().map(|(from,to)| Edge {
                    from : Self::convert_vertex(from),
                    to   : Self::convert_vertex(to),
                });
                let expected_graph = Graph {
                    vertices : expected_vertices.collect(),
                    edges    : expected_edges.collect()
                };
                assert_eq!(graph, expected_graph);
            }

            fn convert_vertex((query_char_index,word_char_index):(usize,usize)) -> Vertex {
                Vertex{query_char_index,word_char_index}
            }
        }

        let classic = Case {
            word : "lalala",
            query : "alA",
            vertices : vec![(0,1),(0,3),(0,5),(1,2),(1,4),(2,3),(2,5)],
            edges    : vec!
                [ ((0,1),(1,2))
                , ((0,1),(1,4))
                , ((0,3),(1,4))
                , ((1,2),(2,3))
                , ((1,2),(2,5))
                , ((1,4),(2,5))
                ]
        };
        let missing_layer = Case {
            word : "laall",
            query : "ala",
            vertices : vec![(0,1),(0,2),(1,3),(1,4)],
            edges    : vec!
                [ ((0,1),(1,3))
                , ((0,1),(1,4))
                , ((0,2),(1,3))
                , ((0,2),(1,4))
                ]
        };
        let empty_word = Case {
            word     : "",
            query    : "ala",
            vertices : vec![],
            edges    : vec![],
        };
        let empty_query = Case {
            word     : "lalala",
            query    : "",
            vertices : vec![],
            edges    : vec![],
        };
        let longer_query = Case {
            word     : "la",
            query    : "ala",
            vertices : vec![(0,1)],
            edges    : vec![],
        };

        for case in vec![classic,missing_layer,empty_query,empty_word,longer_query] {
            case.run()
        }
    }
}
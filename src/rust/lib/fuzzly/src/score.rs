//! Scoring the query-word match.

use crate::prelude::*;

use crate::metric::Metric;
use crate::subsequence_graph;
use crate::SubsequenceGraph;

use std::collections::hash_map::Entry;



// =====================
// === VerticesScore ===
// =====================

/// The score of single vertex in graph.
///
/// The score is a sum of measure of the vertex alone, and the best score of input path.
/// The `max_input_score` is updated during the scoring algorithm run. See the `score_match`
/// function.
#[derive(Copy,Clone,Debug,Default)]
struct VertexScore {
    my_measure      : f32,
    max_input_score : f32,
}

impl VertexScore {
    fn new(my_measure:f32) -> Self {
        let max_input_score = std::f32::NEG_INFINITY;
        VertexScore {my_measure,max_input_score}
    }

    fn update_input_score(&mut self, input_score:f32) {
        self.max_input_score = self.max_input_score.max(input_score);
    }

    fn score(&self) -> f32 {
        self.my_measure + self.max_input_score
    }
}

/// All graph's vertices' scores.
///
/// Used in the `score_match` function.
#[derive(Debug,Default)]
struct VerticesScores(HashMap<subsequence_graph::Vertex,VertexScore>);

impl VerticesScores {
    fn init_vertex(&mut self, vertex:subsequence_graph::Vertex, measure:f32) {
        let Self(scores) = self;
        scores.insert(vertex,VertexScore::new(measure));
    }

    fn update_input_score(&mut self, vertex:subsequence_graph::Vertex, input_score:f32) {
        let Self(scores) = self;
        match scores.entry(vertex) {
            Entry::Occupied(mut entry) => { entry.get_mut().update_input_score(input_score) }
            Entry::Vacant(entry)   => {
                let mut vertex = VertexScore::default();
                vertex.update_input_score(input_score);
                entry.insert(vertex);
            }
        }
    }

    fn get_score(&self, vertex:subsequence_graph::Vertex) -> f32 {
        let Self(scores) = self;
        scores.get(&vertex).map(|v| v.score()).unwrap_or(0.0)
    }
}



// ===================
// === Score Match ===
// ===================

/// Fast-check if the query matches the word.
///
/// This is faster way than calling `score_match(word,query,metric).is_some()`, therefore it's
/// recommended to call this function before scoring when we are not sure if the query actually
/// matches the word.
pub fn matches(word:impl Str, query:impl Str) -> bool {
    let mut query_chars     = query.as_ref().chars();
    let mut next_query_char = query_chars.next();
    for word_char in word.as_ref().chars() {
        if let Some(query_char) = next_query_char {
            if query_char.eq_ignore_ascii_case(&word_char) {
                next_query_char = query_chars.next()
            }
        } else {
            break;
        }
    }
    next_query_char.is_none()
}

/// Score the word-query match.
///
/// The word matches the query if the query is a subsequence of word. However for the peaple some
/// word are better matches for given query. This function scores the match of word with given
/// query basing on some arbitrary metric. Returns `None` if word does not match query. Empty query
/// gives 0.0 score.
///
/// ## Algorithm specification
///
/// In essence, it looks through all possible subsequences of `word` being the `query` and pick the
/// best scoring. Not directly (because there may be a lot of such subsequences), but by building
/// the `SubsequenceGraph` and computing best score for each vertex. See `subsequence_graph` module
/// docs for detailed description of the graph.
pub fn score_match(word:impl Str, query:impl Str, metric:impl Metric) -> Option<f32> {
    let word                  = word.as_ref();
    let query                 = query.as_ref();
    if query.is_empty() {
        Some(0.0)
    } else {
        let last_query_char_index = query.chars().count() - 1;
        let mut scores            = VerticesScores::default();
        let graph                 = SubsequenceGraph::new(word,query);
        for vertex in &graph.vertices {
            let measure = metric.measure_vertex(*vertex,word,query);
            scores.init_vertex(*vertex,measure);
        }
        for vertex in graph.vertices_with_query_char_index(0) {
            scores.update_input_score(*vertex,0.0);
        }
        for edge in &graph.edges {
            let from_score  = scores.get_score(edge.from);
            let input_score = from_score + metric.measure_edge(*edge,word,query);
            scores.update_input_score(edge.to,input_score);
        }
        let end_vertices        = graph.vertices_with_query_char_index(last_query_char_index);
        let end_vertices_scores = end_vertices.map(|v| scores.get_score(*v));
        end_vertices_scores.fold(None, |lhs,rhs| Some(lhs.map_or(rhs, |lhs| lhs.max(rhs))))
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    mod mock_metric {
        use super::*;

        use crate::metric;

        #[derive(Debug,Default)]
        pub struct WordIndex;

        impl Metric for WordIndex {
            fn measure_vertex
            (&self, vertex:subsequence_graph::Vertex, _word:&str, _query:&str) -> f32 {
                vertex.word_char_index as f32
            }

            fn measure_edge(&self, _:subsequence_graph::Edge, _:&str, _:&str) -> f32 { 0.0 }
        }

        #[derive(Debug,Default)]
        pub struct SquareEdgeLength;

        impl Metric for SquareEdgeLength {
            fn measure_vertex(&self, _:subsequence_graph::Vertex, _:&str, _:&str) -> f32 { 0.0 }

            fn measure_edge(&self, edge:subsequence_graph::Edge, _word:&str, _query:&str) -> f32 {
                (edge.to.word_char_index - edge.from.word_char_index).pow(2) as f32
            }
        }

        pub type Sum = metric::Sum<WordIndex,SquareEdgeLength>;
    }

    #[test]
    fn matches_test() {
        assert!( matches("abba", "aba"));
        assert!( matches("abba", "ba" ));
        assert!( matches("abba", ""   ));
        assert!(!matches("abba", "abc"));
        assert!(!matches("abba", "baa"));
        assert!(!matches(""    , "ba" ));
    }

    #[test]
    fn match_scoring() {
        let query = "abc";
        let word  = "aabxbcc";

        assert_eq!(score_match(word,query,mock_metric::WordIndex)       , Some(11.0));
        assert_eq!(score_match(word,query,mock_metric::SquareEdgeLength), Some(20.0));
        assert_eq!(score_match(word,query,mock_metric::Sum::default())  , Some(30.0));
    }

    #[test]
    fn match_scoring_when_does_not_match() {
        let query = "abc";
        let word  = "aabxbyy";
        assert_eq!(score_match(word,query,mock_metric::Sum::default()), None);
    }

    #[test]
    fn match_scoring_corner_cases() {
        let query = "";
        let word  = "any";
        assert_eq!(score_match(word,query,mock_metric::Sum::default()), Some(0.0));
        let query = "any";
        let word  = "";
        assert_eq!(score_match(word,query,mock_metric::Sum::default()), None);
    }
}
//! Library of general data structures.

#![feature(associated_type_bounds)]
#![feature(trait_alias)]

#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

pub use enso_prelude as prelude;

use prelude::*;
use std::collections::hash_map::Entry;
use std::cmp::max;

#[derive(Copy,Clone,Debug,Eq,Hash,PartialEq)]
struct MatchGraphVertex {
    word_char_index  : usize,
    query_char_index : usize,
}

#[derive(Clone,Debug,Default)]
struct MatchGraph {
    sorted_vertices         : Vec<MatchGraphVertex>,
    query_char_index_ranges : Vec<Range<usize>>,
    score                   : f32,
}

impl MatchGraph {
    fn new(word:impl Str, query:impl Str) -> Self {
        let mut this                = MatchGraph::default();
        let mut current_range_start = 0;
        for (i,query_ch) in query.as_ref().chars().enumerate() {
            let previous    = i.checked_sub(1);
            let unreachable = previous.map_or(0, |prev_i| this.vertices_with_query_char_index(prev_i).first().map_or(word.len(), |v| v.word_char_index + 1));
            for (j,word_ch) in word.as_ref().chars().enumerate().skip(unreachable) {
                if query_ch == word_ch {
                    this.sorted_vertices.push(MatchGraphVertex {
                        word_char_index  : j,
                        query_char_index : i,
                    });
                }
            }
            this.query_char_index_ranges.push(current_range_start..this.sorted_vertices.len());
            current_range_start = this.sorted_vertices.len();
        }
        this
    }

    fn enumerate_vertices_with_query_char_index(&self, index:usize) -> impl Iter<(usize,&MatchGraphVertex)> {
        let start..end = self.query_char_index_ranges[index];
        self.sorted_vertices.iter().enumerate().skip(start).take(end-start)
    }

    fn edge_iter(&self) -> EdgeIterator {
        EdgeIterator {
            graph             : &self,
            next_edge_indices : self.first_edge_in_subgraph(0..self.sorted_vertices.len())
        }
    }

    fn first_edge_in_subgraph(&self, mut vertices_range:Range<usize>) -> Option<(usize,usize)> {
        vertices_range.find_map(|from_index| {
            let from     = &self.sorted_vertices[from_index];
            let to_index = self.enumerate_vertices_with_query_char_index(from.query_char_index+1).find_map(|(i,to)| {
                (to.word_char_index > from.word_char_index).and_option(Some(i))
            })?;
            Some((from_index+1, to_index))
        })
    }
}

struct EdgeIterator<'a> {
    graph             : &'a MatchGraph,
    next_edge_indices : Option<(usize,usize)>,
}

impl<'a> Iterator for EdgeIterator<'a> {
    type Item = (&'a MatchGraphVertex, &'a MatchGraphVertex);

    fn next(&mut self) -> Option<Self::Item> {
        let (from_index,to_index) = self.next_edge_indices?;
        let from                  = &self.graph.sorted_vertices[from_index];
        let to                    = &self.graph.sorted_vertices[to_index];
        self.next_edge_indices = || {
            let next_to = self.graph.sorted_vertices.get(to_index + 1).filter(|vertex| vertex.query_char_index == from.query_char_index + 1);
            if next_to.is_none() {
                // We should pick next from
                self.graph.first_edge_in_subgraph(from_index+1..self.graph.sorted_vertices.len())
            } else {
                Some((from_index,to_index+1))
            }
        }();
        Some((from,to))
    }
}

trait Metrics {
    fn measure_beginning(vertex:&MatchGraphVertex, word:&str, query:&str) -> f32;
    fn measure_ending(vertex:&MatchGraphVertex, word:&str, query:&str) -> f32;
    fn measure_edge(edge:(&MatchGraphVertex,&MatchGraphVertex), word:&str, query:&str) -> f32;
}

fn score_match(word:impl Str, query:impl Str, metrics:impl Metrics) -> Option<f32> {
    let word                  = word.as_ref();
    let query                 = query.as_ref();
    let last_query_char_index = query.len().checked_sub(1)?;
    let mut scores            = HashMap::<MatchGraphVertex,f32>::new();
    let graph                 = MatchGraph::new(word,query);
    for (_,vertex) in graph.enumerate_vertices_with_query_char_index(0) {
        scores.insert(*vertex,metrics.measure_beginning(vertex,word,query));
    }
    for (from,to) in graph.edge_iter() {
        let from_score         = scores.get(from).cloned().unwrap_or(0.0);
        let to_score_candidate = from_score + metrics.measure_edge((from,to),word,query);
        match scores.entry(*to) {
            Entry::Occupied(mut entry) => entry.insert(max(*entry.get(),to_score_candidate)),
            Entry::Vacant  (mut entry) => entry.insert(to_score_candidate),
        }
    }
    graph.enumerate_vertices_with_query_char_index(last_query_char_index).map(|(_,vertex)| {
        scores.get(vertex).cloned().unwrap_or(0.0) + metrics.measure_ending(vertex,word,query)
    }).max()
}
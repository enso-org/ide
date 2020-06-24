/// Conflict-free replicated data type (CRDT) implementation. A data structure which can be
/// replicated across multiple computers in a network, where the replicas can be updated
/// independently and concurrently without coordination between the replicas, and where it is always
/// mathematically possible to resolve inconsistencies which might result.
///
/// More info: https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type

pub use xi_rope::engine::Engine;
pub use xi_rope::engine::RevId;
pub use xi_rope::engine::RevToken;

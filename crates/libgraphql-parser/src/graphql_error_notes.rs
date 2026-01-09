use crate::GraphQLSourceSpan;
use crate::SmallVec;

/// Type alias for error notes. Each note is a message with an optional span
/// indicating where the note applies.
/// Uses SmallVec since most errors have 0-2 notes.
pub type GraphQLErrorNotes = SmallVec<[(String, Option<GraphQLSourceSpan>); 2]>;

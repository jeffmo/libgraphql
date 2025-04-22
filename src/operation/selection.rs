use crate::DirectiveAnnotation;
use crate::loc;
use crate::operation::NamedFragmentRef;
use crate::operation::SelectionSet;
use crate::Value;
use crate::types::NamedGraphQLTypeRef;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Selection<'schema> {
    Field(FieldSelection<'schema>),
    InlineFragment(InlineFragmentSelection<'schema>),
    NamedFragment(NamedFragmentSelection<'schema>),
}

#[derive(Debug)]
pub struct FieldSelection<'schema> {
    pub(super) alias: Option<String>,
    pub(super) arguments: HashMap<String, Value>,
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) name: String,
    pub(super) position: loc::FilePosition,
    pub(super) selection_set: SelectionSet<'schema>,
}

#[derive(Debug)]
pub struct InlineFragmentSelection<'schema> {
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) position: loc::FilePosition,
    pub(super) selection_set: SelectionSet<'schema>,
    pub(super) type_condition: Option<NamedGraphQLTypeRef>,
}

#[derive(Debug)]
pub struct NamedFragmentSelection<'schema> {
    pub(super) directives: Vec<DirectiveAnnotation>,
    pub(super) fragment: NamedFragmentRef<'schema>,
    pub(super) position: loc::FilePosition,
}

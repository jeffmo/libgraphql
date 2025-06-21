use crate::ast;
use crate::loc;
use crate::schema::Schema;
use crate::Value;
use crate::types::Directive;
use crate::types::NamedDirectiveRef;
use std::collections::BTreeMap;
use std::path::Path;

/// Represents a
/// [directive annotation](https://spec.graphql.org/October2021/#sec-Language.Directives)
/// placed somewhere within a [`GraphQLType`](crate::types::GraphQLType),
/// [`Mutation`](crate::operation::Mutation),
/// [`Query`](crate::operation::Query), or
/// [`Subscription`](crate::operation::Subscription).
///
/// A [`DirectiveAnnotation`] can be thought of as a "pointer" to some
/// [`Directive`] paired with a set of named arguments ([`Value`]s).
#[derive(Clone, Debug, PartialEq)]
pub struct DirectiveAnnotation {
    pub(crate) args: BTreeMap<String, Value>,
    pub(crate) directive_ref: NamedDirectiveRef,
}
impl DirectiveAnnotation {
    /// A map from ParameterName -> [`Value`] for all arguments passed to this
    /// [`DirectiveAnnotation`].
    ///
    /// This returns a [`BTreeMap`] to guarantee that map entries retain the same
    /// ordering as the order of arguments passed to this directive annotation.
    pub fn args(&self) -> &BTreeMap<String, Value> {
        &self.args
    }

    /// The [`SchemaDefLocation`](loc::SchemaDefLocation) indicating where this
    /// annotation was specified within some
    /// [`GraphQLType`](crate::types::GraphQLType),
    /// [`Mutation`](crate::operation::Mutation),
    /// [`Query`](crate::operation::Query),
    /// or [`Subscription`](crate::operation::Subscription).
    pub fn def_location(&self) -> &loc::SchemaDefLocation {
        self.directive_ref.def_location()
    }

    /// The [`Directive`] type for which this annotation refers to.
    pub fn directive_type<'schema>(
        &self,
        schema: &'schema Schema,
    ) -> &'schema Directive {
        self.directive_ref.deref(schema).unwrap()
    }

    pub(crate) fn from_ast<P: AsRef<Path>>(
        file_path: P,
        ast_annots: &[ast::operation::Directive],
    ) -> Vec<Self> {
        let file_path = file_path.as_ref();
        let mut annots = vec![];
        for ast_annot in ast_annots {
            let mut args = BTreeMap::new();
            for (arg_name, arg_val) in ast_annot.arguments.iter() {
                args.insert(arg_name.to_string(), Value::from_ast(
                    arg_val,
                    loc::FilePosition::from_pos(
                        file_path,
                        ast_annot.position,
                    ),
                ));
            }

            annots.push(DirectiveAnnotation {
                args,
                directive_ref: NamedDirectiveRef::new(
                    &ast_annot.name,
                    loc::SchemaDefLocation::Schema(loc::FilePosition::from_pos(
                        file_path,
                        ast_annot.position,
                    )),
                ),
            });
        }
        annots
    }
}


use crate::DirectiveAnnotation;
use crate::loc;
use crate::NamedRef;
use crate::schema::SchemaBuilder;
use crate::schema::SchemaBuildError;
use crate::schema::schema_builder::GraphQLOperationType;
use crate::schema::schema_builder::NamedTypeDefLocation;
use crate::types::GraphQLType;
use indexmap::IndexMap;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

mod basics {
    use super::*;

    #[test]
    fn build_without_load() -> Result<()> {
        let schema = SchemaBuilder::new()
            .build();

        assert!(schema.is_err());
        assert!(matches!(
            schema.unwrap_err(),
            SchemaBuildError::NoQueryOperationTypeDefined,
        ));

        Ok(())
    }

    #[test]
    fn load_all_empty_operation_types_in_single_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, concat!(
                "type Mutation\n",
                "type Query\n",
                "type Subscription",
            ))?
            .build()
            .unwrap();

        assert_eq!(schema.directive_defs.len(), 4);
        assert_eq!(schema.types.len(), 8);

        // Empty `Mutation` type
        let mutation_type_ref = schema.mutation_type.clone().unwrap();
        let mut_type = mutation_type_ref.deref(&schema).unwrap();
        let mutation_obj_type = mut_type.as_object().expect("type is an object type");
        assert_eq!(mutation_obj_type.def_location(), &loc::SourceLocation::Schema);
        assert!(mutation_obj_type.directives().is_empty());
        assert_eq!(mutation_obj_type.fields().keys().collect::<Vec<_>>(), vec![
            &"__typename".to_string(),
        ]);
        assert_eq!(mutation_obj_type.name(), "Mutation");

        // Empty `Query` type
        let query_type_ref = schema.query_type.clone();
        let query_type = query_type_ref.deref(&schema).unwrap();
        let query_obj_type = query_type.as_object().expect("type is an object type");
        assert_eq!(query_obj_type.def_location(), &loc::SourceLocation::Schema);
        assert!(query_obj_type.directives().is_empty());
        assert_eq!(mutation_obj_type.fields().keys().collect::<Vec<_>>(), vec![
            &"__typename".to_string(),
        ]);
        assert_eq!(query_obj_type.name(), "Query");

        // Empty `Subscription` type
        let subscription_type_ref = schema.subscription_type.clone().unwrap();
        let subscription_type = subscription_type_ref.deref(&schema).unwrap();
        let subscription_obj_type = subscription_type.as_object().expect("type is an object type");
        assert_eq!(subscription_obj_type.def_location(), &loc::SourceLocation::Schema);
        assert!(subscription_obj_type.directives().is_empty());
        assert_eq!(subscription_obj_type.fields().keys().collect::<Vec<_>>(), vec![
            &"__typename".to_string(),
        ]);
        assert_eq!(subscription_obj_type.name(), "Subscription");

        Ok(())
    }

    #[test]
    fn load_empty_query_type_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, "type Query")?
            .build()
            .unwrap();

        assert_eq!(schema.directive_defs.len(), 4);
        assert!(schema.mutation_type.is_none());
        assert!(schema.subscription_type.is_none());
        assert_eq!(schema.types.len(), 6);

        let query_type_ref = schema.query_type.clone();
        let query_type = query_type_ref.deref(&schema).unwrap();
        let query_obj_type = query_type.as_object().expect("type is an object");
        assert_eq!(query_obj_type.name(), "Query");
        assert_eq!(query_obj_type.def_location(), &loc::SourceLocation::Schema);
        assert!(query_obj_type.directives().is_empty());
        assert_eq!(query_obj_type.fields().keys().collect::<Vec<_>>(), vec![
            &"__typename".to_string(),
        ]);

        Ok(())
    }

    #[test]
    fn load_invalid_schema_syntax() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, "this is not valid syntax");

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::ParseError {
                file: None,
                err: "schema parse error: Parse error at \
                      1:1\nUnexpected `this[Name]`\nExpected schema, \
                      extend, scalar, type, interface, union, \
                      enum, input or directive\n".to_string(),
            },
        );

        Ok(())
    }

    #[test]
    fn load_multiple_schema_definition_with_query_and_duplicate_mutation_operations() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, concat!(
                "type TQuery\n",
                "type TMutation1\n",
                "schema {\n",
                "  mutation: TMutation1,\n",
                "  query: TQuery,\n",
                "}\n",
            ))?
            .load_str(None, concat!(
                "type TMutation2\n",
                "schema {\n",
                "  mutation: TMutation2,\n",
                "}\n",
            ));

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::DuplicateOperationDefinition {
                operation: GraphQLOperationType::Mutation,
                location1: NamedTypeDefLocation {
                    def_location: loc::SourceLocation::Schema,
                    type_name: "TMutation1".to_string(),
                },
                location2: NamedTypeDefLocation {
                    def_location: loc::SourceLocation::Schema,
                    type_name: "TMutation2".to_string(),
                },
            },
        );

        Ok(())
    }

    #[test]
    fn load_multiple_schema_definition_with_query_and_duplicate_subscription_operations() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, concat!(
                "type TQuery\n",
                "type TSubscription1\n",
                "schema {\n",
                "  subscription: TSubscription1,\n",
                "  query: TQuery,\n",
                "}\n",
            ))?
            .load_str(None, concat!(
                "type TSubscription2\n",
                "schema {\n",
                "  subscription: TSubscription2,\n",
                "}\n",
            ));

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::DuplicateOperationDefinition {
                operation: GraphQLOperationType::Subscription,
                location1: NamedTypeDefLocation {
                    def_location: loc::SourceLocation::Schema,
                    type_name: "TSubscription1".to_string(),
                },
                location2: NamedTypeDefLocation {
                    def_location: loc::SourceLocation::Schema,
                    type_name: "TSubscription2".to_string(),
                },
            },
        );

        Ok(())
    }

    #[test]
    fn load_only_empty_mutation_type_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, "type Mutation")?
            .build();

        assert!(schema.is_err());
        assert!(matches!(
            schema.unwrap_err(),
            SchemaBuildError::NoQueryOperationTypeDefined,
        ));

        Ok(())
    }

    /// Section 3.3.1 of the GraphQL spec states that a schema must always define a Query operation type:
    /// https://spec.graphql.org/October2021/#sec-Root-Operation-Types
    #[test]
    fn load_only_empty_subscription_type_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, "type Subscription")?
            .build();

        assert!(schema.is_err());
        assert!(matches!(
            schema.unwrap_err(),
            SchemaBuildError::NoQueryOperationTypeDefined,
        ));

        Ok(())
    }

    #[test]
    fn load_schema_definition_with_query_and_mutation_operations() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, concat!(
                "type TQuery\n",
                "type TMutation\n",
                "schema {\n",
                "  mutation: TMutation,\n",
                "  query: TQuery,\n",
                "}\n",
            ))?
            .build()
            .unwrap();

        assert!(schema.subscription_type.is_none());

        let actual_query_type = schema.query_type.deref(&schema).unwrap();
        let expected_query_type = schema.types.get("TQuery").unwrap();
        assert_eq!(actual_query_type, expected_query_type);

        let actual_mutation_typeref = schema.mutation_type.as_ref().unwrap();
        let actual_mutation_type = actual_mutation_typeref.deref(&schema).unwrap();
        let expected_mutation_type = schema.types.get("TMutation").unwrap();
        assert_eq!(actual_mutation_type, expected_mutation_type);

        Ok(())
    }

    #[test]
    fn load_schema_definition_with_query_and_subscription_operations() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, concat!(
                "type TQuery\n",
                "type TSubscription\n",
                "schema {\n",
                "  query: TQuery,\n",
                "  subscription: TSubscription,\n",
                "}\n",
            ))?
            .build()
            .unwrap();

        assert!(schema.mutation_type.is_none());

        let actual_query_type = schema.query_type.deref(&schema).unwrap();
        let expected_query_type = schema.types.get("TQuery").unwrap();
        assert_eq!(actual_query_type, expected_query_type);

        let actual_subscription_typeref = schema.subscription_type.as_ref().unwrap();
        let actual_subscription_type = actual_subscription_typeref.deref(&schema).unwrap();
        let expected_subscription_type = schema.types.get("TSubscription").unwrap();
        assert_eq!(actual_subscription_type, expected_subscription_type);

        Ok(())
    }

    #[test]
    fn load_schema_definition_with_no_operation_types() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, "schema {}")?
            .build();

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::NoQueryOperationTypeDefined,
        );

        Ok(())
    }

    #[test]
    fn load_schema_definition_with_only_mutation_operation() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, concat!(
                "type Foo\n",
                "schema {\n",
                "  mutation: Foo,\n",
                "}\n",
            ))?
            .build();

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::NoQueryOperationTypeDefined,
        );

        Ok(())
    }

    #[test]
    fn load_schema_definition_with_only_query_operation_type() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, concat!(
                "type Foo\n",
                "schema {\n",
                "  query: Foo,\n",
                "}\n",
            ))?
            .build()
            .unwrap();

        let actual_query_type = schema.query_type.deref(&schema).unwrap();
        let expected_query_type = schema.types.get("Foo").unwrap();
        assert_eq!(actual_query_type, expected_query_type);

        Ok(())
    }

    #[test]
    fn load_schema_definition_with_only_subscription_operation() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_str(None, concat!(
                "type Foo\n",
                "schema {\n",
                "  subscription: Foo,\n",
                "}\n",
            ))?
            .build();

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::NoQueryOperationTypeDefined,
        );

        Ok(())
    }

    #[test]
    fn load_valid_schema_def_with_path() -> Result<()> {
        let schema_path = PathBuf::from("test://example_file");
        let schema = SchemaBuilder::new()
            .load_str(
                Some(schema_path.as_path()),
                "type Query",
            )?
            .build()?;

        assert_eq!(schema.types.len(), 6);

        Ok(())
    }

    #[test]
    fn load_invalid_schema_def_with_path() -> Result<()> {
        let schema_path = PathBuf::from("test://example_file");
        let schema = SchemaBuilder::new().load_str(
            Some(schema_path.as_path()),
            concat!(
                "type Query\n",
                "NOPE_SYNTAX_ERROR\n",
            ),
        );

        match schema {
            Err(SchemaBuildError::ParseError {
                file,
                ..
            }) => assert_eq!(
                file,
                Some(schema_path),
            ),

            _ => panic!(
                "Expected ParseError but got {schema:?}"
            ),
        };

        Ok(())
    }
}

mod object_types {
    use super::*;

    #[test]
    fn conflicting_type_name_in_separate_loads() -> Result<()> {
        let builder = SchemaBuilder::new()
            .load_str(None, "type Query type Foo")?
            .load_str(None, "type Foo");

        assert_eq!(builder.unwrap_err(), SchemaBuildError::DuplicateTypeDefinition {
            type_name: "Foo".to_string(),
            def1: loc::SourceLocation::Schema,
            def2: loc::SourceLocation::Schema,
        });

        Ok(())
    }

    #[test]
    fn conflicting_type_name_in_single_load() -> Result<()> {
        let builder = SchemaBuilder::new()
            .load_str(None, "type Query type Foo type Foo");

        assert_eq!(builder.unwrap_err(), SchemaBuildError::DuplicateTypeDefinition {
            type_name: "Foo".to_string(),
            def1: loc::SourceLocation::Schema,
            def2: loc::SourceLocation::Schema,
        });

        Ok(())
    }

    mod implementing_interfaces {
        use super::*;

        #[test]
        fn single_interface() -> Result<()> {
            // TODO

            Ok(())
        }

        #[test]
        fn multiple_interfaces() -> Result<()> {
            // TODO
            Ok(())
        }
    }

    mod with_object_directives {
        use super::*;

        #[test]
        fn single_builtin_directive() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "type Foo @deprecated\n",
                ))?
                .build()?;

            let foo_type = schema.types.get("Foo").unwrap();
            let type_data = match foo_type {
                GraphQLType::Object(type_data) => type_data,
                _ => panic!(
                    concat!(
                        "Schema defines an object type with name `Foo`, but ",
                        "SchemaBuilder constructed `{:?}`.",
                    ),
                    foo_type,
                ),
            };

            assert_eq!(type_data.def_location(), &loc::SourceLocation::Schema);
            assert_eq!(type_data.directives(), &vec![
                DirectiveAnnotation {
                    arguments: IndexMap::new(),
                    directive_ref: NamedRef::new("deprecated", loc::SourceLocation::Schema),
                },
            ]);
            assert_eq!(type_data.fields().keys().collect::<Vec<_>>(), vec![
                &"__typename".to_string(),
            ]);
            assert_eq!(type_data.name(), "Foo");

            Ok(())
        }

        #[test]
        fn single_custom_directive() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "type Foo @customDirective\n",
                ))?
                .build()?;

            let foo_type = schema.types.get("Foo").unwrap();
            let type_data = match foo_type {
                GraphQLType::Object(type_data) => type_data,
                _ => panic!(
                    concat!(
                        "Schema defines an object type with name `Foo`, but ",
                        "SchemaBuilder constructed `{:?}`.",
                    ),
                    foo_type,
                ),
            };

            assert_eq!(type_data.def_location(), &loc::SourceLocation::Schema);
            assert_eq!(type_data.directives(), &vec![
                DirectiveAnnotation {
                    arguments: IndexMap::new(),
                    directive_ref: NamedRef::new("customDirective", loc::SourceLocation::Schema),
                },
            ]);
            assert_eq!(type_data.fields().keys().collect::<Vec<_>>(), vec![
                &"__typename".to_string(),
            ]);
            assert_eq!(type_data.name(), "Foo");

            Ok(())
        }

        #[test]
        fn multiple_directives() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "type Foo @customDirective @deprecated\n",
                ))?
                .build()?;

            let foo_type = schema.types.get("Foo").unwrap();
            let type_data = match foo_type {
                GraphQLType::Object(type_data) => type_data,
                _ => panic!(
                    concat!(
                        "Schema defines an object type with name `Foo`, but ",
                        "SchemaBuilder constructed `{:?}`.",
                    ),
                    foo_type,
                ),
            };

            assert_eq!(type_data.def_location(), &loc::SourceLocation::Schema);
            assert_eq!(type_data.directives(), &vec![
                DirectiveAnnotation {
                    arguments: IndexMap::new(),
                    directive_ref: NamedRef::new("customDirective", loc::SourceLocation::Schema),
                },
                DirectiveAnnotation {
                    arguments: IndexMap::new(),
                    directive_ref: NamedRef::new("deprecated", loc::SourceLocation::Schema),
                },
            ]);
            assert_eq!(type_data.fields().keys().collect::<Vec<_>>(), vec![
                &"__typename".to_string(),
            ]);
            assert_eq!(type_data.name(), "Foo");

            Ok(())
        }
    }

    mod with_field_arg_directives {
        // TODO
    }

    mod with_field_directives {
        // TODO
    }

    mod with_fields {
        use super::*;

        #[test]
        fn multiple_object_fields() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "type Bar {\n",
                    "  barField: String!,\n",
                    "}\n",
                    "type Baz {\n",
                    "  bazField: Int,\n",
                    "}\n",
                    "type Foo {\n",
                    "  bar: Bar,\n",
                    "  baz: Baz!,\n",
                    "}\n",
                ))?
                .build()?;

            let foo_type = schema.types.get("Foo").unwrap();
            let type_data = match foo_type {
                GraphQLType::Object(type_data) => type_data,
                _ => panic!(
                    concat!(
                        "Schema defines an object type with name `Foo`, but ",
                        "SchemaBuilder constructed `{:?}`.",
                    ),
                    foo_type,
                ),
            };

            assert_eq!(type_data.def_location(), &loc::SourceLocation::Schema);
            assert_eq!(type_data.directives(), &vec![]);

            assert_eq!(type_data.name(), "Foo");

            let bar_field = type_data.fields().get("bar").unwrap();
            assert_eq!(bar_field.def_location(), &loc::SourceLocation::Schema);

            let bar_field_type_annot =
                bar_field.type_annotation().as_named_annotation().unwrap();
            assert_eq!(bar_field_type_annot.ref_location(), &loc::SourceLocation::Schema);
            assert!(bar_field_type_annot.nullable());

            let bar_field_type =
                bar_field_type_annot.graphql_type(&schema)
                    .as_object()
                    .unwrap();
            assert_eq!(bar_field_type.name(), "Bar");

            let baz_field = type_data.fields().get("baz").unwrap();
            assert_eq!(baz_field.def_location(), &loc::SourceLocation::Schema);

            let baz_field_type_annot =
                baz_field.type_annotation().as_named_annotation().unwrap();
            assert_eq!(baz_field_type_annot.ref_location(), &loc::SourceLocation::Schema);

            let baz_field_type =
                baz_field_type_annot.graphql_type(&schema)
                    .as_object()
                    .unwrap();
            assert_eq!(baz_field_type.name(), "Baz");

            Ok(())
        }

        #[test]
        fn multiple_scalar_fields() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "type Foo {\n",
                    "  stringField: String,\n",
                    "  intField: Int!,\n",
                    "}\n",
                ))?
                .build()?;

            let foo_type = schema.types.get("Foo").unwrap();
            let type_data = match foo_type {
                GraphQLType::Object(type_data) => type_data,
                _ => panic!(
                    concat!(
                        "Schema defines an object type with name `Foo`, but ",
                        "SchemaBuilder constructed `{:?}`.",
                    ),
                    foo_type,
                ),
            };

            assert_eq!(type_data.def_location(), &loc::SourceLocation::Schema);
            assert_eq!(type_data.directives(), &vec![]);

            let string_field = type_data.fields().get("stringField").unwrap();
            assert_eq!(string_field.def_location(), &loc::SourceLocation::Schema);

            let string_field_type_annot =
                string_field.type_annotation().as_named_annotation().unwrap();
            assert_eq!(string_field_type_annot.ref_location(), &loc::SourceLocation::Schema);
            assert!(string_field_type_annot.nullable());

            assert!(matches!(
                string_field_type_annot.graphql_type(&schema),
                &GraphQLType::String,
            ));

            let int_field = type_data.fields().get("intField").unwrap();
            assert_eq!(int_field.def_location(), &loc::SourceLocation::Schema);

            let int_field_type_annot =
                int_field.type_annotation().as_named_annotation().unwrap();
            assert_eq!(int_field_type_annot.ref_location(), &loc::SourceLocation::Schema);
            assert!(!int_field_type_annot.nullable());

            assert!(matches!(
                int_field_type_annot.graphql_type(&schema),
                &GraphQLType::Int,
            ));

            assert_eq!(type_data.name(), "Foo");

            Ok(())
        }

        mod with_params {
            // TODO
        }

        mod with_field_directives {
            // TODO
        }
    }

    mod extensions {
        use super::*;

        #[test]
        fn extension_adds_preexisting_field_name() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "type Foo {\n",
                    "  foo_field: Boolean,\n",
                    "}\n",
                    "extend type Foo {\n",
                    "  foo_field: Boolean,\n",
                    "}",
                ));

            assert!(schema.is_err());
            assert_eq!(
                schema.unwrap_err(),
                SchemaBuildError::DuplicateFieldNameDefinition {
                    type_name: "Foo".to_string(),
                    field_name: "foo_field".to_string(),
                    field_def1: loc::SourceLocation::Schema,
                    field_def2: loc::SourceLocation::Schema,
                },
            );

            Ok(())
        }

        #[test]
        fn extension_adds_valid_field() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "type Foo\n",
                    "extend type Foo @extended_type_directive {\n",
                    "  extended_field: Boolean @extended_field_directive\n",
                    "}",
                ))?
                .build()?;

            // Has only the 1 field
            let obj_type = schema.types.get("Foo").unwrap();
            let obj_type = obj_type.as_object().expect("type is an object");
            assert_eq!(obj_type.fields().keys().collect::<Vec<_>>(), vec![
                &"__typename".to_string(),
                &"extended_field".to_string(),
            ]);

            // Type has directive added at type-extension site
            assert_eq!(obj_type.directives(), &vec![
                DirectiveAnnotation {
                    arguments: IndexMap::new(),
                    directive_ref: NamedRef::new(
                        "extended_type_directive",
                        loc::SourceLocation::Schema,
                    ),
                }
            ]);

            // Foo.extended_field is nullable
            let extended_field = obj_type.fields().get("extended_field").unwrap();
            assert!(extended_field.type_annotation().nullable());

            // Foo.extended_field's def_location is correct
            assert_eq!(extended_field.def_location(), &loc::SourceLocation::Schema);

            // Foo.extended_field is a bool type
            let extended_field_type =
                extended_field.type_annotation()
                    .inner_named_type_ref()
                    .deref(&schema)
                    .unwrap();
            assert!(matches!(extended_field_type, GraphQLType::Bool));

            Ok(())
        }

        #[test]
        fn object_extension_of_nonobject_type() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "enum Foo { Value1 }\n",
                    "extend type Foo {\n",
                    "  foo_field: Boolean,\n",
                    "}",
                ));

            assert!(schema.is_err());
            let error = schema.unwrap_err();
            match error {
                SchemaBuildError::InvalidExtensionType {
                    schema_type: GraphQLType::Enum(enum_type),
                    extension_location: _,
                } => {
                    assert_eq!(enum_type.def_location(), &loc::SourceLocation::Schema);
                    assert_eq!(enum_type.directives(), &vec![]);
                    assert_eq!(enum_type.name(), "Foo");

                    let values = enum_type.values();
                    assert_eq!(values.len(), 1);

                    let value1 = values.get("Value1").unwrap();
                    assert_eq!(value1.def_location(), &loc::SourceLocation::Schema);
                    assert_eq!(value1.directives(), &vec![]);
                    assert_eq!(value1.name(), "Value1");
                },

                _ => panic!(
                    "Expected `SchemaBuildError::InvalidExtensionType` but \
                    found {error}.",
                ),
            }

            Ok(())
        }

        #[test]
        fn object_extension_of_undefined_type() -> Result<()> {
            let schema = SchemaBuilder::new()
                .load_str(None, concat!(
                    "type Query\n",
                    "extend type Foo {\n",
                    "  foo_field: Boolean,\n",
                    "}",
                ))?
                .build();

            assert!(schema.is_err());
            assert_eq!(
                schema.unwrap_err(),
                SchemaBuildError::ExtensionOfUndefinedType {
                    type_name: "Foo".to_string(),
                    extension_location: loc::SourceLocation::Schema,
                },
            );

            Ok(())
        }
    }
}

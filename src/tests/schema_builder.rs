use crate::loc;
use crate::named_ref::NamedRef;
use crate::schema_builder::GraphQLOperation;
use crate::schema_builder::SchemaBuilder;
use crate::schema_builder::SchemaBuildError;
use crate::schema_builder::NamedTypeFilePosition;
use crate::types::GraphQLEnumType;
use crate::types::GraphQLType;
use std::collections::HashMap;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

mod build_operations {
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
            .load_from_str(None, concat!(
                "type Mutation\n",
                "type Query\n",
                "type Subscription",
            ))?
            .build()
            .unwrap();

        assert_eq!(schema.directives.len(), 4);
        assert_eq!(schema.types.len(), 8);

        // Empty `Mutation` type
        let mutation_type_ref = schema.mutation_type.clone().unwrap();
        let mut_type = mutation_type_ref.deref(&schema).unwrap();
        let mutation_obj_type = mut_type.unwrap_object();
        assert_eq!(mutation_obj_type.def_location, loc::FilePosition {
            col: 1,
            file: PathBuf::from("str://0"),
            line: 1,
        });
        assert!(mutation_obj_type.directives.is_empty());
        assert!(mutation_obj_type.fields.is_empty());
        assert_eq!(mutation_obj_type.name, "Mutation");

        // Empty `Query` type
        let query_type_ref = schema.query_type.clone();
        let query_type = query_type_ref.deref(&schema).unwrap();
        let query_obj_type = query_type.unwrap_object();
        assert_eq!(query_obj_type.def_location, loc::FilePosition {
            col: 1,
            file: PathBuf::from("str://0"),
            line: 2,
        });
        assert!(query_obj_type.directives.is_empty());
        assert!(query_obj_type.fields.is_empty());
        assert_eq!(query_obj_type.name, "Query");

        // Empty `Subscription` type
        let subscription_type_ref = schema.subscription_type.clone().unwrap();
        let subscription_type = subscription_type_ref.deref(&schema).unwrap();
        let subscription_obj_type = subscription_type.unwrap_object();
        assert_eq!(subscription_obj_type.def_location, loc::FilePosition {
            col: 1,
            file: PathBuf::from("str://0"),
            line: 3,
        });
        assert!(subscription_obj_type.directives.is_empty());
        assert!(subscription_obj_type.fields.is_empty());
        assert_eq!(subscription_obj_type.name, "Subscription");

        Ok(())
    }

    #[test]
    fn load_empty_query_type_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, "type Query")?
            .build()
            .unwrap();

        assert_eq!(schema.directives.len(), 4);
        assert!(schema.mutation_type.is_none());
        assert!(schema.subscription_type.is_none());
        assert_eq!(schema.types.len(), 6);

        let query_type_ref = schema.query_type.clone();
        let query_type = query_type_ref.deref(&schema).unwrap();
        let query_obj_type = query_type.unwrap_object();
        assert_eq!(query_obj_type.name, "Query");
        assert_eq!(query_obj_type.def_location, loc::FilePosition {
            col: 1,
            file: PathBuf::from("str://0"),
            line: 1,
        });
        assert!(query_obj_type.directives.is_empty());
        assert!(query_obj_type.fields.is_empty());

        Ok(())
    }

    #[test]
    fn load_only_empty_mutation_type_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, "type Mutation")?
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
            .load_from_str(None, "type Subscription")?
            .build();

        assert!(schema.is_err());
        assert!(matches!(
            schema.unwrap_err(),
            SchemaBuildError::NoQueryOperationTypeDefined,
        ));

        Ok(())
    }

    #[test]
    fn load_multiple_schema_definition_with_query_and_duplicate_mutation_operations() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, concat!(
                "type TQuery\n",
                "type TMutation1\n",
                "schema {\n",
                "  mutation: TMutation1,\n",
                "  query: TQuery,\n",
                "}\n",
            ))?
            .load_from_str(None, concat!(
                "type TMutation2\n",
                "schema {\n",
                "  mutation: TMutation2,\n",
                "}\n",
            ));

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::DuplicateOperationDefinition {
                operation: GraphQLOperation::Mutation,
                location1: NamedTypeFilePosition {
                    def_location: loc::FilePosition {
                        col: 1,
                        file: PathBuf::from("str://0"),
                        line: 3,
                    },
                    type_name: "TMutation1".to_string(),
                },
                location2: NamedTypeFilePosition {
                    def_location: loc::FilePosition {
                        col: 1,
                        file: PathBuf::from("str://1"),
                        line: 2,
                    },
                    type_name: "TMutation2".to_string(),
                },
            },
        );

        Ok(())
    }

    #[test]
    fn load_multiple_schema_definition_with_query_and_duplicate_subscription_operations() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, concat!(
                "type TQuery\n",
                "type TSubscription1\n",
                "schema {\n",
                "  subscription: TSubscription1,\n",
                "  query: TQuery,\n",
                "}\n",
            ))?
            .load_from_str(None, concat!(
                "type TSubscription2\n",
                "schema {\n",
                "  subscription: TSubscription2,\n",
                "}\n",
            ));

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::DuplicateOperationDefinition {
                operation: GraphQLOperation::Subscription,
                location1: NamedTypeFilePosition {
                    def_location: loc::FilePosition {
                        col: 1,
                        file: PathBuf::from("str://0"),
                        line: 3,
                    },
                    type_name: "TSubscription1".to_string(),
                },
                location2: NamedTypeFilePosition {
                    def_location: loc::FilePosition {
                        col: 1,
                        file: PathBuf::from("str://1"),
                        line: 2,
                    },
                    type_name: "TSubscription2".to_string(),
                },
            },
        );

        Ok(())
    }

    #[test]
    fn load_schema_definition_with_query_and_mutation_operations() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, concat!(
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
            .load_from_str(None, concat!(
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
            .load_from_str(None, "schema {}")?
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
            .load_from_str(None, concat!(
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
            .load_from_str(None, concat!(
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
            .load_from_str(None, concat!(
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
}

mod build_object_types {
    use super::*;

    #[test]
    fn conflicting_type_name_in_single_load() -> Result<()> {
        let builder = SchemaBuilder::new()
            .load_from_str(None, "type Query type Foo type Foo");

        assert_eq!(builder.unwrap_err(), SchemaBuildError::DuplicateTypeDefinition {
            type_name: "Foo".to_string(),
            def1: loc::SchemaDefLocation::Schema(loc::FilePosition {
                col: 12,
                file: PathBuf::from("str://0"),
                line: 1,
            }),
            def2: loc::SchemaDefLocation::Schema(loc::FilePosition {
                col: 21,
                file: PathBuf::from("str://0"),
                line: 1,
            }),
        });

        Ok(())
    }

    #[test]
    fn conflicting_type_name_in_separate_loads() -> Result<()> {
        let builder = SchemaBuilder::new()
            .load_from_str(None, "type Query type Foo")?
            .load_from_str(None, "type Foo");

        assert_eq!(builder.unwrap_err(), SchemaBuildError::DuplicateTypeDefinition {
            type_name: "Foo".to_string(),
            def1: loc::SchemaDefLocation::Schema(loc::FilePosition {
                col: 12,
                file: PathBuf::from("str://0"),
                line: 1,
            }),
            def2: loc::SchemaDefLocation::Schema(loc::FilePosition {
                col: 1,
                file: PathBuf::from("str://1"),
                line: 1,
            }),
        });

        Ok(())
    }

    #[test]
    fn type_extension_adds_preexisting_field_name() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, concat!(
                "type Query\n",
                "type Foo {\n",
                "  foo_field: Boolean,\n",
                "}\n",
                "extend type Foo {\n",
                "  foo_field: Boolean,\n",
                "}",
            ))?
            .build();

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::DuplicateFieldNameDefinition {
                type_name: "Foo".to_string(),
                field_name: "foo_field".to_string(),
                field_def1: loc::SchemaDefLocation::Schema(loc::FilePosition {
                    col: 3,
                    file: PathBuf::from("str://0"),
                    line: 3,
                }),
                field_def2: loc::SchemaDefLocation::Schema(loc::FilePosition {
                    col: 3,
                    file: PathBuf::from("str://0"),
                    line: 6,
                }),
            },
        );

        Ok(())
    }

    #[test]
    fn type_extension_adds_valid_field() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, concat!(
                "type Query\n",
                "type Foo\n",
                "extend type Foo @extended_type_directive {\n",
                "  extended_field: Boolean @extended_field_directive\n",
                "}",
            ))?
            .build()?;

        // Has only the 1 field
        let obj_type = schema.types.get("Foo").unwrap();
        let obj_type = obj_type.unwrap_object();
        assert_eq!(obj_type.fields.len(), 1);

        // Type has directive added at type-extension site
        assert_eq!(obj_type.directives, vec![
            NamedRef::new("extended_type_directive".to_string(), loc::FilePosition {
                col: 17,
                file: PathBuf::from("str://0"),
                line: 3,
            }),
        ]);

        // Foo.extended_field is nullable
        let extended_field = obj_type.fields.get("extended_field").unwrap();
        assert!(extended_field.type_ref.is_nullable());

        // Foo.extended_field's def_location is correct
        assert_eq!(extended_field.def_location, loc::SchemaDefLocation::Schema(
            loc::FilePosition {
                col: 3,
                file: PathBuf::from("str://0"),
                line: 4,
            },
        ));

        // Foo.extended_field is a bool type
        let extended_field_type =
            extended_field.type_ref
                .extract_named_type_ref()
                .deref(&schema)
                .unwrap();
        assert!(matches!(extended_field_type, GraphQLType::Bool));

        Ok(())
    }

    #[test]
    fn type_extension_of_nonobject_type() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, concat!(
                "type Query\n",
                "enum Foo\n",
                "extend type Foo {\n",
                "  foo_field: Boolean,\n",
                "}",
            ))?
            .build();

        assert!(schema.is_err());
        assert_eq!(
            schema.unwrap_err(),
            SchemaBuildError::InvalidExtensionType {
                schema_type: GraphQLType::Enum(GraphQLEnumType {
                   def_location: loc::FilePosition {
                       col: 1,
                       file: PathBuf::from("str://0"),
                       line: 2,
                   },
                   directives: vec![],
                   name: "Foo".to_string(),
                   variants: HashMap::new(),
                }),
                extension_loc: loc::FilePosition {
                    col: 8,
                    file: PathBuf::from("str://0"),
                    line: 3,
                },
            },
        );

        Ok(())
    }

    #[test]
    fn type_extension_of_undefined_type() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, concat!(
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
                extension_type_loc: loc::FilePosition {
                    col: 8,
                    file: PathBuf::from("str://0"),
                    line: 2,
                },
            },
        );

        Ok(())
    }
}

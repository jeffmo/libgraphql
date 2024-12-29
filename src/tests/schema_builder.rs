use crate::loc;
use crate::named_ref::NamedRef;
use crate::schema_builder::SchemaBuilder;
use crate::schema_builder::SchemaBuildError;
use crate::types::GraphQLType;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, SchemaBuildError>;

mod build_operations {
    use super::*;

    #[test]
    fn build_all_empty_operation_types_single_str() -> Result<()> {
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
        let query_type_ref = schema.query_type.clone().unwrap();
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
    fn build_empty_query_type_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, "type Query")?
            .build()
            .unwrap();

        assert_eq!(schema.directives.len(), 4);
        assert!(schema.mutation_type.is_none());
        assert!(schema.subscription_type.is_none());
        assert_eq!(schema.types.len(), 6);

        let query_type_ref = schema.query_type.clone().unwrap();
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
    fn build_empty_mutation_type_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, "type Mutation")?
            .build()
            .unwrap();

        assert_eq!(schema.directives.len(), 4);
        assert!(schema.query_type.is_none());
        assert!(schema.subscription_type.is_none());
        assert_eq!(schema.types.len(), 6);

        let mutation_type_ref = schema.mutation_type.clone().unwrap();
        let mut_type = mutation_type_ref.deref(&schema).unwrap();
        let mut_obj_type = mut_type.unwrap_object();
        assert_eq!(mut_obj_type.name, "Mutation");
        assert_eq!(mut_obj_type.def_location, loc::FilePosition {
            col: 1,
            file: PathBuf::from("str://0"),
            line: 1,
        });
        assert!(mut_obj_type.directives.is_empty());
        assert!(mut_obj_type.fields.is_empty());

        Ok(())
    }

    #[test]
    fn build_empty_subscription_type_str() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, "type Subscription")?
            .build()
            .unwrap();

        assert_eq!(schema.directives.len(), 4);
        assert!(schema.query_type.is_none());
        assert!(schema.mutation_type.is_none());
        assert_eq!(schema.types.len(), 6);

        let subscription_type_ref = schema.subscription_type.clone().unwrap();
        let sub_type = subscription_type_ref.deref(&schema).unwrap();
        let sub_obj_type = sub_type.unwrap_object();
        assert_eq!(sub_obj_type.def_location, loc::FilePosition {
            col: 1,
            file: PathBuf::from("str://0"),
            line: 1,
        });
        assert!(sub_obj_type.directives.is_empty());
        assert!(sub_obj_type.fields.is_empty());
        assert_eq!(sub_obj_type.name, "Subscription");

        Ok(())
    }

    #[test]
    fn build_without_load() -> Result<()> {
        let schema = SchemaBuilder::new()
            .build();

        assert!(schema.is_err());
        assert!(matches!(
            schema.unwrap_err(),
            SchemaBuildError::NoOperationTypesDefined,
        ));

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
    fn type_extension() -> Result<()> {
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
                line: 1,
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
                line: 2,
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
}

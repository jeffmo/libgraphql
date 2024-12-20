use crate::loc;
use crate::schema_builder::SchemaBuilder;
use crate::schema_builder::SchemaBuildError;
use crate::types::GraphQLType;
use crate::types::GraphQLTypeRef;

type Result<T> = std::result::Result<T, SchemaBuildError>;

#[test]
fn empty_builder() -> Result<()> {
    let schema = SchemaBuilder::new()
        .build();

    assert!(schema.is_err());
    assert!(matches!(
        schema.unwrap_err(),
        SchemaBuildError::NoOperationTypesDefined,
    ));

    Ok(())
}

#[test]
fn empty_query_operation_only() -> Result<()> {
    let schema = SchemaBuilder::new()
        .load_from_str(None, "type Query")?
        .build()
        .unwrap();

    assert_eq!(schema.directives.len(), 4);
    assert!(schema.mutation_type.is_none());
    assert!(schema.subscription_type.is_none());
    assert_eq!(schema.types.len(), 6);

    let query_type_ref = schema.query_type.clone().unwrap();
    match query_type_ref.deref(&schema).unwrap() {
        GraphQLType::Object {
            def_location,
            directives,
            fields,
            name,
        } => {
            assert_eq!(*def_location, loc::FilePosition {
                col: 1,
                file: None,
                line: 1,
            });
            assert!(directives.is_empty());
            assert!(fields.is_empty());
            assert_eq!(name, "Query");
        },

        unexpected_type => assert!(
            false,
            "Unexpected GraphQLType set for Schema.query_type: {:#?}",
            unexpected_type,
        ),
    }

    Ok(())
}

#[test]
fn empty_mutation_operation_only() -> Result<()> {
    let schema = SchemaBuilder::new()
        .load_from_str(None, "type Mutation")?
        .build()
        .unwrap();

    assert_eq!(schema.directives.len(), 4);
    assert!(schema.query_type.is_none());
    assert!(schema.subscription_type.is_none());
    assert_eq!(schema.types.len(), 6);

    let mutation_type_ref = schema.mutation_type.clone().unwrap();
    match mutation_type_ref.deref(&schema).unwrap() {
        GraphQLType::Object {
            def_location,
            directives,
            fields,
            name,
        } => {
            assert_eq!(*def_location, loc::FilePosition {
                col: 1,
                file: None,
                line: 1,
            });
            assert!(directives.is_empty());
            assert!(fields.is_empty());
            assert_eq!(name, "Mutation");
        },

        unexpected_type => assert!(
            false,
            "Unexpected GraphQLType set for Schema.mutation_type: {:#?}",
            unexpected_type,
        ),
    }

    Ok(())
}

#[test]
fn empty_subscription_operation_only() -> Result<()> {
    let schema = SchemaBuilder::new()
        .load_from_str(None, "type Subscription")?
        .build()
        .unwrap();

    assert_eq!(schema.directives.len(), 4);
    assert!(schema.query_type.is_none());
    assert!(schema.mutation_type.is_none());
    assert_eq!(schema.types.len(), 6);

    let subscription_type_ref = schema.subscription_type.clone().unwrap();
    match subscription_type_ref.deref(&schema).unwrap() {
        GraphQLType::Object {
            def_location,
            directives,
            fields,
            name,
        } => {
            assert_eq!(*def_location, loc::FilePosition {
                col: 1,
                file: None,
                line: 1,
            });
            assert!(directives.is_empty());
            assert!(fields.is_empty());
            assert_eq!(name, "Subscription");
        },

        unexpected_type => assert!(
            false,
            "Unexpected GraphQLType set for Schema.subscription_type: {:#?}",
            unexpected_type,
        ),
    }

    Ok(())
}

mod object_types {
    use super::*;

    #[test]
    fn conflicting_type_name() -> Result<()> {
        let builder = SchemaBuilder::new()
            .load_from_str(None, "type Query type Foo")?
            .load_from_str(None, "type Foo");

        assert_eq!(builder.unwrap_err(), SchemaBuildError::DuplicateTypeDefinition {
            type_name: "Foo".to_string(),
            def1: loc::SchemaDefLocation::Schema(loc::FilePosition {
                col: 12,
                file: None,
                line: 1,
            }),
            def2: loc::SchemaDefLocation::Schema(loc::FilePosition {
                col: 1,
                file: None,
                line: 1,
            }),
        });

        Ok(())
    }

    #[test]
    fn single_object_type_extension() -> Result<()> {
        let schema = SchemaBuilder::new()
            .load_from_str(None, "type Query type Foo")?
            .load_from_str(None, "extend type Foo { extended_field: Boolean }")?
            .build()?;

        let obj_type = schema.types.get("Foo").unwrap();
        let fields = match obj_type {
            GraphQLType::Object { fields, .. } => fields,
            _ => panic!("Invalid type built for extended object type: {:?}", obj_type),
        };
        assert_eq!(fields.len(), 1);

        let extended_field = fields.get("extended_field").unwrap();
        let extended_field_type = match &extended_field.type_ref {
            GraphQLTypeRef::Named {
                nullable,
                type_ref,
            } => {
                assert_eq!(*nullable, true);
                type_ref.deref(&schema).unwrap()
            },

            _ => panic!("Invalid field type built for extended object field: {:?}", extended_field.type_ref),
        };

        assert!(matches!(extended_field_type, GraphQLType::Bool));

        Ok(())
    }
}

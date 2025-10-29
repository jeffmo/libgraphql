use crate::graphql_parse_error::{GraphQLParseError, GraphQLParseErrorKind, GraphQLParseErrors};
use crate::graphql_token_stream::GraphQLTokenStream;
use crate::rust_to_graphql_token_adapter::{GraphQLToken, RustToGraphQLTokenAdapter};
use libgraphql_core::ast;
use proc_macro2::Span;

type ParseResult<T> = Result<T, GraphQLParseError>;

/// A GraphQLToken -> GraphQL Schema Document Parser
///
/// This parser consumes tokens from a `GraphQLTokenStream` and builds a GraphQL
/// schema AST that is 1:1 compatible with `graphql_parser::schema::parse_schema()`.
/// It provides:
/// - Precise error reporting with span information
/// - Error recovery to collect multiple errors in one pass
pub(crate) struct GraphQLSchemaParser {
    tokens: GraphQLTokenStream,
    errors: GraphQLParseErrors,
}

impl GraphQLSchemaParser {
    /// Creates a new parser from a token adapter
    pub fn new(adapter: RustToGraphQLTokenAdapter) -> Self {
        Self {
            tokens: GraphQLTokenStream::new(adapter),
            errors: GraphQLParseErrors::new(),
        }
    }

    /// Parses a complete GraphQL schema document
    ///
    /// Returns either a valid Document or a collection of errors encountered
    /// during parsing. The parser attempts to recover from errors and continue
    /// parsing to collect as many errors as possible.
    pub fn parse_document(mut self) -> Result<ast::schema::Document, GraphQLParseErrors> {
        let mut definitions = Vec::new();

        while !self.tokens.is_at_end() {
            match self.parse_definition() {
                Ok(def) => definitions.push(def),
                Err(err) => {
                    self.errors.add(err);
                    self.recover_to_next_definition();
                }
            }
        }

        if self.errors.has_errors() {
            Err(self.errors)
        } else {
            Ok(ast::schema::Document { definitions })
        }
    }

    /// Parses a single top-level definition
    fn parse_definition(&mut self) -> ParseResult<ast::schema::Definition> {
        // Peek at the current token to determine what kind of definition this is
        let (token, span) = match self.tokens.peek() {
            Some(t) => t.clone(),
            None => return Err(self.unexpected_eof_error(vec!["definition keyword".to_string()])),
        };

        match token {
            GraphQLToken::Name(ref name) => match name.as_str() {
                "type" => self.parse_object_type_definition(),
                "interface" => self.parse_interface_type_definition(),
                "union" => self.parse_union_type_definition(),
                "enum" => self.parse_enum_type_definition(),
                "scalar" => self.parse_scalar_type_definition(),
                "input" => self.parse_input_object_type_definition(),
                "directive" => self.parse_directive_definition(),
                "schema" => self.parse_schema_definition(),
                "extend" => self.parse_type_extension(),
                _ => Err(GraphQLParseError::new(
                    format!("Unexpected keyword '{name}'. Expected a type definition keyword (type, interface, union, enum, scalar, input, directive, schema, or extend)."),
                    span,
                    GraphQLParseErrorKind::UnexpectedToken {
                        expected: vec!["type".to_string(), "interface".to_string(), "union".to_string()],
                        found: name.clone(),
                    },
                )),
            },
            _ => Err(GraphQLParseError::new(
                format!("Expected a definition keyword, but found {}", self.token_description(&token)),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec!["definition keyword".to_string()],
                    found: self.token_description(&token),
                },
            )),
        }
    }

    /// Parses a scalar type definition: `scalar Name @directives`
    fn parse_scalar_type_definition(&mut self) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("scalar")?;
        let (name, _name_span) = self.expect_name_value()?;
        let directives = self.parse_directives()?;

        Ok(ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Scalar(ast::schema::ScalarType {
                position: self.span_to_pos(start_span),
                description: None,
                name,
                directives,
            })
        ))
    }

    /// Parses an object type definition: `type Name implements Interfaces @directives { fields }`
    fn parse_object_type_definition(&mut self) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("type")?;
        let (name, _name_span) = self.expect_name_value()?;
        let implements_interfaces = self.parse_implements_interfaces()?;
        let directives = self.parse_directives()?;
        let fields = self.parse_fields_definition()?;

        Ok(ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Object(ast::schema::ObjectType {
                position: self.span_to_pos(start_span),
                description: None,
                name,
                implements_interfaces,
                directives,
                fields,
            })
        ))
    }

    /// Parses an interface type definition
    /// `interface Name implements Interfaces @directives { fields }`
    fn parse_interface_type_definition(
        &mut self
    ) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("interface")?;
        let (name, _name_span) = self.expect_name_value()?;
        let implements_interfaces = self.parse_implements_interfaces()?;
        let directives = self.parse_directives()?;
        let fields = self.parse_fields_definition()?;

        Ok(ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Interface(
                ast::schema::InterfaceType {
                    position: self.span_to_pos(start_span),
                    description: None,
                    name,
                    implements_interfaces,
                    directives,
                    fields,
                }
            )
        ))
    }

    /// Parses a union type definition
    /// `union Name @directives = Type1 | Type2 | Type3`
    fn parse_union_type_definition(
        &mut self,
    ) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("union")?;
        let (name, _name_span) = self.expect_name_value()?;
        let directives = self.parse_directives()?;

        // Union types must have = followed by types
        self.expect_punctuator("=")?;

        // Optional leading |
        let _ = self.skip_if_punctuator("|");

        let mut types = Vec::new();
        loop {
            let (type_name, _) = self.expect_name_value()?;
            types.push(type_name);

            if !self.skip_if_punctuator("|") {
                break;
            }
        }

        Ok(ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Union(ast::schema::UnionType {
                position: self.span_to_pos(start_span),
                description: None,
                name,
                directives,
                types,
            }),
        ))
    }

    /// Parses an enum type definition
    /// `enum Name @directives { VALUE1 VALUE2 }`
    fn parse_enum_type_definition(
        &mut self,
    ) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("enum")?;
        let (name, _name_span) = self.expect_name_value()?;
        let directives = self.parse_directives()?;

        let open_brace_span = self.expect_punctuator("{")?;

        let mut values = Vec::new();
        while !self.tokens.check_punctuator("}") {
            if self.tokens.is_at_end() {
                return Err(self.unclosed_delimiter_error(
                    "{",
                    open_brace_span,
                ));
            }
            values.push(self.parse_enum_value_definition()?);
        }

        self.expect_punctuator("}")?;

        Ok(ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::Enum(ast::schema::EnumType {
                position: self.span_to_pos(start_span),
                description: None,
                name,
                directives,
                values,
            }),
        ))
    }

    /// Parses an enum value definition: `VALUE @directives`
    fn parse_enum_value_definition(
        &mut self,
    ) -> ParseResult<ast::schema::EnumValue> {
        let (name, name_span) = self.expect_name_value()?;
        let directives = self.parse_directives()?;

        Ok(ast::schema::EnumValue {
            position: self.span_to_pos(name_span),
            description: None,
            name,
            directives,
        })
    }

    /// Parses an input object type definition
    /// `input Name @directives { fields }`
    fn parse_input_object_type_definition(
        &mut self,
    ) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("input")?;
        let (name, _name_span) = self.expect_name_value()?;
        let directives = self.parse_directives()?;

        let open_brace_span = self.expect_punctuator("{")?;

        let mut fields = Vec::new();
        while !self.tokens.check_punctuator("}") {
            if self.tokens.is_at_end() {
                return Err(self.unclosed_delimiter_error(
                    "{",
                    open_brace_span,
                ));
            }
            fields.push(self.parse_input_value_definition()?);
        }

        self.expect_punctuator("}")?;

        Ok(ast::schema::Definition::TypeDefinition(
            ast::schema::TypeDefinition::InputObject(
                ast::schema::InputObjectType {
                    position: self.span_to_pos(start_span),
                    description: None,
                    name,
                    directives,
                    fields,
                },
            ),
        ))
    }

    /// Parses a directive definition
    /// `directive @name(args) repeatable on LOCATION | LOCATION`
    fn parse_directive_definition(
        &mut self,
    ) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("directive")?;
        self.expect_punctuator("@")?;
        let (name, _name_span) = self.expect_name_value()?;

        let arguments = if self.tokens.check_punctuator("(") {
            self.parse_arguments_definition()?
        } else {
            Vec::new()
        };

        // Optional "repeatable" keyword
        let repeatable = self.skip_if_name("repeatable");

        self.expect_name("on")?;

        // Optional leading |
        let _ = self.skip_if_punctuator("|");

        let mut locations = Vec::new();
        loop {
            let (location_name, loc_span) = self.expect_name_value()?;
            let location = self.parse_directive_location(
                &location_name,
                loc_span,
            )?;
            locations.push(location);

            if !self.skip_if_punctuator("|") {
                break;
            }
        }

        Ok(ast::schema::Definition::DirectiveDefinition(
            ast::schema::DirectiveDefinition {
                position: self.span_to_pos(start_span),
                description: None,
                name,
                arguments,
                repeatable,
                locations,
            },
        ))
    }

    /// Parses a directive location name into the enum
    fn parse_directive_location(
        &self,
        name: &str,
        span: Span,
    ) -> ParseResult<ast::schema::DirectiveLocation> {
        use ast::schema::DirectiveLocation;
        match name {
            "QUERY" => Ok(DirectiveLocation::Query),
            "MUTATION" => Ok(DirectiveLocation::Mutation),
            "SUBSCRIPTION" => Ok(DirectiveLocation::Subscription),
            "FIELD" => Ok(DirectiveLocation::Field),
            "FRAGMENT_DEFINITION" => {
                Ok(DirectiveLocation::FragmentDefinition)
            }
            "FRAGMENT_SPREAD" => {
                Ok(DirectiveLocation::FragmentSpread)
            }
            "INLINE_FRAGMENT" => {
                Ok(DirectiveLocation::InlineFragment)
            }
            "SCHEMA" => Ok(DirectiveLocation::Schema),
            "SCALAR" => Ok(DirectiveLocation::Scalar),
            "OBJECT" => Ok(DirectiveLocation::Object),
            "FIELD_DEFINITION" => {
                Ok(DirectiveLocation::FieldDefinition)
            }
            "ARGUMENT_DEFINITION" => {
                Ok(DirectiveLocation::ArgumentDefinition)
            }
            "INTERFACE" => Ok(DirectiveLocation::Interface),
            "UNION" => Ok(DirectiveLocation::Union),
            "ENUM" => Ok(DirectiveLocation::Enum),
            "ENUM_VALUE" => Ok(DirectiveLocation::EnumValue),
            "INPUT_OBJECT" => Ok(DirectiveLocation::InputObject),
            "INPUT_FIELD_DEFINITION" => {
                Ok(DirectiveLocation::InputFieldDefinition)
            }
            "VARIABLE_DEFINITION" => {
                Ok(DirectiveLocation::VariableDefinition)
            }
            _ => Err(GraphQLParseError::new(
                format!("Invalid directive location '{name}'"),
                span,
                GraphQLParseErrorKind::InvalidDirectiveLocation,
            )),
        }
    }

    /// Parses a schema definition
    /// `schema @directives { query: Query mutation: Mutation }`
    fn parse_schema_definition(
        &mut self,
    ) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("schema")?;
        let directives = self.parse_directives()?;

        let open_brace_span = self.expect_punctuator("{")?;

        let mut query = None;
        let mut mutation = None;
        let mut subscription = None;

        while !self.tokens.check_punctuator("}") {
            if self.tokens.is_at_end() {
                return Err(self.unclosed_delimiter_error(
                    "{",
                    open_brace_span,
                ));
            }

            let (operation_type, op_span) = self.expect_name_value()?;
            self.expect_punctuator(":")?;
            let (type_name, _) = self.expect_name_value()?;

            match operation_type.as_str() {
                "query" => query = Some(type_name),
                "mutation" => mutation = Some(type_name),
                "subscription" => subscription = Some(type_name),
                _ => {
                    return Err(GraphQLParseError::new(
                        format!(
                            "Invalid operation type '{}' in schema",
                            operation_type
                        ),
                        op_span,
                        GraphQLParseErrorKind::InvalidSyntax,
                    ))
                }
            }
        }

        self.expect_punctuator("}")?;

        Ok(ast::schema::Definition::SchemaDefinition(
            ast::schema::SchemaDefinition {
                position: self.span_to_pos(start_span),
                directives,
                query,
                mutation,
                subscription,
            },
        ))
    }

    /// Parses a type extension
    /// `extend type Name ...` or `extend interface Name ...` etc.
    fn parse_type_extension(
        &mut self,
    ) -> ParseResult<ast::schema::Definition> {
        let start_span = self.expect_name("extend")?;

        let (token, span) = match self.tokens.peek() {
            Some(t) => t.clone(),
            None => {
                return Err(self.unexpected_eof_error(vec![
                    "type".to_string(),
                    "interface".to_string(),
                    "union".to_string(),
                    "enum".to_string(),
                    "input".to_string(),
                    "scalar".to_string(),
                    "schema".to_string(),
                ]))
            }
        };

        match token {
            GraphQLToken::Name(ref keyword) => match keyword.as_str() {
                "type" => self.parse_object_type_extension(start_span),
                "interface" => {
                    self.parse_interface_type_extension(start_span)
                }
                "union" => self.parse_union_type_extension(start_span),
                "enum" => self.parse_enum_type_extension(start_span),
                "input" => {
                    self.parse_input_object_type_extension(start_span)
                }
                "scalar" => {
                    self.parse_scalar_type_extension(start_span)
                }
                "schema" => self.parse_schema_extension(start_span),
                _ => Err(GraphQLParseError::new(
                    format!(
                        "Expected type extension keyword, found '{}'",
                        keyword
                    ),
                    span,
                    GraphQLParseErrorKind::InvalidSyntax,
                )),
            },
            _ => Err(GraphQLParseError::new(
                "Expected type extension keyword".to_string(),
                span,
                GraphQLParseErrorKind::InvalidSyntax,
            )),
        }
    }

    /// Parses an implements interfaces clause: `implements Interface1 & Interface2`
    fn parse_implements_interfaces(&mut self) -> ParseResult<Vec<String>> {
        if !self.tokens.check_name("implements") {
            return Ok(Vec::new());
        }

        self.tokens.next(); // consume "implements"

        let mut interfaces = Vec::new();
        loop {
            let (interface_name, _) = self.expect_name_value()?;
            interfaces.push(interface_name);

            if !self.tokens.check_punctuator("&") {
                break;
            }
            self.tokens.next(); // consume "&"
        }

        Ok(interfaces)
    }

    /// Parses a fields definition block: `{ field1: Type, field2: Type }`
    fn parse_fields_definition(&mut self) -> ParseResult<Vec<ast::schema::Field>> {
        self.expect_punctuator("{")?;

        let mut fields = Vec::new();

        while !self.tokens.check_punctuator("}") && !self.tokens.is_at_end() {
            match self.parse_field_definition() {
                Ok(field) => fields.push(field),
                Err(err) => {
                    self.errors.add(err);
                    self.recover_to_next_field();
                }
            }
        }

        self.expect_punctuator("}")?;
        Ok(fields)
    }

    /// Parses a single field definition: `name(args): Type @directives`
    fn parse_field_definition(&mut self) -> ParseResult<ast::schema::Field> {
        let (name, name_span) = self.expect_name_value()?;
        let arguments = self.parse_arguments_definition()?;
        self.expect_punctuator(":")?;
        let field_type = self.parse_type()?;
        let directives = self.parse_directives()?;

        Ok(ast::schema::Field {
            position: self.span_to_pos(name_span),
            description: None,
            name,
            arguments,
            field_type,
            directives,
        })
    }

    /// Parses field arguments definition: `(arg1: Type, arg2: Type = default)`
    fn parse_arguments_definition(&mut self) -> ParseResult<Vec<ast::schema::InputValue>> {
        if !self.tokens.check_punctuator("(") {
            return Ok(Vec::new());
        }

        self.tokens.next(); // consume "("

        let mut arguments = Vec::new();

        while !self.tokens.check_punctuator(")") && !self.tokens.is_at_end() {
            let arg = self.parse_input_value_definition()?;
            arguments.push(arg);

            // Optional comma between arguments
            if self.tokens.check_punctuator(",") {
                self.tokens.next();
            }
        }

        self.expect_punctuator(")")?;
        Ok(arguments)
    }

    /// Parses an input value definition: `name: Type = defaultValue @directives`
    fn parse_input_value_definition(&mut self) -> ParseResult<ast::schema::InputValue> {
        let (name, name_span) = self.expect_name_value()?;
        self.expect_punctuator(":")?;
        let value_type = self.parse_type()?;

        let default_value = if self.tokens.check_punctuator("=") {
            self.tokens.next(); // consume "="
            Some(self.parse_value()?)
        } else {
            None
        };

        let directives = self.parse_directives()?;

        Ok(ast::schema::InputValue {
            position: self.span_to_pos(name_span),
            description: None,
            name,
            value_type,
            default_value,
            directives,
        })
    }

    /// Parses a type reference: `Type`, `Type!`, `[Type]`, `[Type!]!`, etc.
    fn parse_type(&mut self) -> ParseResult<ast::schema::Type> {
        let mut ty = self.parse_type_base()?;

        // Handle non-null modifier
        if self.tokens.check_punctuator("!") {
            self.tokens.next();
            ty = ast::schema::Type::NonNullType(Box::new(ty));
        }

        Ok(ty)
    }

    /// Parses the base type (either named type or list type)
    fn parse_type_base(&mut self) -> ParseResult<ast::schema::Type> {
        if self.tokens.check_punctuator("[") {
            self.tokens.next(); // consume "["
            let inner = self.parse_type()?;
            self.expect_punctuator("]")?;
            Ok(ast::schema::Type::ListType(Box::new(inner)))
        } else {
            let (name, _) = self.expect_name_value()?;
            Ok(ast::schema::Type::NamedType(name))
        }
    }

    /// Parses directives: `@directive1 @directive2(arg: value)`
    fn parse_directives(&mut self) -> ParseResult<Vec<ast::operation::Directive>> {
        let mut directives = Vec::new();

        while self.tokens.check_punctuator("@") {
            directives.push(self.parse_directive()?);
        }

        Ok(directives)
    }

    /// Parses a single directive: `@name(arguments)`
    fn parse_directive(&mut self) -> ParseResult<ast::operation::Directive> {
        let at_span = self.expect_punctuator("@")?;
        let (name, _) = self.expect_name_value()?;
        let arguments = self.parse_directive_arguments()?;

        Ok(ast::operation::Directive {
            position: self.span_to_pos(at_span),
            name,
            arguments,
        })
    }

    /// Parses directive arguments: `(arg1: value1, arg2: value2)`
    fn parse_directive_arguments(&mut self) -> ParseResult<Vec<(String, ast::Value)>> {
        if !self.tokens.check_punctuator("(") {
            return Ok(Vec::new());
        }

        self.tokens.next(); // consume "("

        let mut arguments = Vec::new();

        while !self.tokens.check_punctuator(")") && !self.tokens.is_at_end() {
            let (arg_name, _) = self.expect_name_value()?;
            self.expect_punctuator(":")?;
            let value = self.parse_value()?;
            arguments.push((arg_name, value));

            // Optional comma
            if self.tokens.check_punctuator(",") {
                self.tokens.next();
            }
        }

        self.expect_punctuator(")")?;
        Ok(arguments)
    }

    /// Parses a GraphQL value (int, float, string, boolean, null, enum, list, object)
    fn parse_value(&mut self) -> ParseResult<ast::Value> {
        let (token, span) = match self.tokens.peek() {
            Some(t) => t.clone(),
            None => return Err(self.unexpected_eof_error(vec!["value".to_string()])),
        };

        match token {
            GraphQLToken::IntValue(i) => {
                self.tokens.next();
                Ok(ast::Value::Int(ast::Number::from(i as i32)))
            }
            GraphQLToken::FloatValue(f) => {
                self.tokens.next();
                Ok(ast::Value::Float(f))
            }
            GraphQLToken::StringValue(s) => {
                self.tokens.next();
                Ok(ast::Value::String(s))
            }
            GraphQLToken::Name(ref name) => {
                self.tokens.next();
                match name.as_str() {
                    "true" => Ok(ast::Value::Boolean(true)),
                    "false" => Ok(ast::Value::Boolean(false)),
                    "null" => Ok(ast::Value::Null),
                    _ => Ok(ast::Value::Enum(name.clone())),
                }
            }
            GraphQLToken::Punctuator(ref p) if p == "[" => {
                self.parse_list_value()
            }
            GraphQLToken::Punctuator(ref p) if p == "{" => {
                self.parse_object_value()
            }
            _ => Err(GraphQLParseError::new(
                format!("Expected a value, but found {}", self.token_description(&token)),
                span,
                GraphQLParseErrorKind::InvalidValue {
                    details: format!("Found {}", self.token_description(&token)),
                },
            )),
        }
    }

    /// Parses a list value: `[value1, value2, ...]`
    fn parse_list_value(&mut self) -> ParseResult<ast::Value> {
        self.expect_punctuator("[")?;

        let mut values = Vec::new();

        while !self.tokens.check_punctuator("]") && !self.tokens.is_at_end() {
            values.push(self.parse_value()?);

            // Optional comma
            if self.tokens.check_punctuator(",") {
                self.tokens.next();
            }
        }

        self.expect_punctuator("]")?;
        Ok(ast::Value::List(values))
    }

    /// Parses an object value: `{ key1: value1, key2: value2 }`
    fn parse_object_value(&mut self) -> ParseResult<ast::Value> {
        use std::collections::BTreeMap;

        self.expect_punctuator("{")?;

        let mut fields = BTreeMap::new();

        while !self.tokens.check_punctuator("}") && !self.tokens.is_at_end() {
            let (key, _) = self.expect_name_value()?;
            self.expect_punctuator(":")?;
            let value = self.parse_value()?;
            fields.insert(key, value);

            // Optional comma
            if self.tokens.check_punctuator(",") {
                self.tokens.next();
            }
        }

        self.expect_punctuator("}")?;
        Ok(ast::Value::Object(fields))
    }

    // ========== Helper Methods ==========

    /// Expects a specific name keyword and returns its span
    fn expect_name(&mut self, expected: &str) -> ParseResult<Span> {
        let (token, span) = self.tokens.next()
            .ok_or_else(|| self.unexpected_eof_error(vec![expected.to_string()]))?;

        match token {
            GraphQLToken::Name(name) if name == expected => Ok(span),
            GraphQLToken::Name(name) => Err(GraphQLParseError::new(
                format!("Expected '{expected}', but found '{name}'"),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![expected.to_string()],
                    found: name,
                },
            )),
            _ => Err(GraphQLParseError::new(
                format!("Expected '{expected}', but found {}", self.token_description(&token)),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![expected.to_string()],
                    found: self.token_description(&token),
                },
            )),
        }
    }

    /// Expects any name and returns its value and span
    fn expect_name_value(&mut self) -> ParseResult<(String, Span)> {
        let (token, span) = self.tokens.next()
            .ok_or_else(|| self.unexpected_eof_error(vec!["identifier".to_string()]))?;

        match token {
            GraphQLToken::Name(name) => Ok((name, span)),
            _ => Err(GraphQLParseError::new(
                format!("Expected an identifier, but found {}", self.token_description(&token)),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec!["identifier".to_string()],
                    found: self.token_description(&token),
                },
            )),
        }
    }

    /// Expects a specific punctuator and returns its span
    fn expect_punctuator(&mut self, expected: &str) -> ParseResult<Span> {
        let (token, span) = self.tokens.next()
            .ok_or_else(|| self.unexpected_eof_error(vec![expected.to_string()]))?;

        match token {
            GraphQLToken::Punctuator(p) if p == expected => Ok(span),
            GraphQLToken::Punctuator(p) => Err(GraphQLParseError::new(
                format!("Expected '{expected}', but found '{p}'"),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![expected.to_string()],
                    found: p,
                },
            )),
            _ => Err(GraphQLParseError::new(
                format!("Expected '{expected}', but found {}", self.token_description(&token)),
                span,
                GraphQLParseErrorKind::UnexpectedToken {
                    expected: vec![expected.to_string()],
                    found: self.token_description(&token),
                },
            )),
        }
    }

    /// Creates an unexpected EOF error
    fn unexpected_eof_error(&self, expected: Vec<String>) -> GraphQLParseError {
        GraphQLParseError::new(
            format!("Unexpected end of input. Expected: {}", expected.join(", ")),
            self.tokens.current_span(),
            GraphQLParseErrorKind::UnexpectedEof { expected },
        )
    }

    /// Converts a Span to a graphql_parser::Pos
    fn span_to_pos(&self, _span: Span) -> ast::AstPos {
        // For now, we use a default position since proc_macro2::Span
        // doesn't provide line/column information in a stable way
        ast::AstPos::default()
    }

    /// Returns a human-readable description of a token
    fn token_description(&self, token: &GraphQLToken) -> String {
        match token {
            GraphQLToken::Name(n) => format!("identifier '{n}'"),
            GraphQLToken::Punctuator(p) => format!("'{p}'"),
            GraphQLToken::IntValue(i) => format!("integer {i}"),
            GraphQLToken::FloatValue(f) => format!("float {f}"),
            GraphQLToken::StringValue(s) => format!("string \"{s}\""),
        }
    }

    /// Skips a specific name if present, returns true if skipped
    fn skip_if_name(&mut self, name: &str) -> bool {
        if self.tokens.check_name(name) {
            self.tokens.next();
            true
        } else {
            false
        }
    }

    /// Skips a specific punctuator if present, returns true if skipped
    fn skip_if_punctuator(&mut self, punct: &str) -> bool {
        if self.tokens.check_punctuator(punct) {
            self.tokens.next();
            true
        } else {
            false
        }
    }

    /// Creates an unclosed delimiter error
    fn unclosed_delimiter_error(
        &self,
        delimiter: &str,
        opening_span: Span,
    ) -> GraphQLParseError {
        GraphQLParseError::with_spans(
            format!("Unclosed delimiter '{delimiter}'"),
            vec![opening_span, self.tokens.current_span()],
            GraphQLParseErrorKind::UnclosedDelimiter {
                delimiter: delimiter.to_string(),
                opening_span_available: true,
            },
        )
    }

    // ========== Type Extension Parsers ==========

    /// Parses an object type extension
    fn parse_object_type_extension(
        &mut self,
        extend_span: Span,
    ) -> ParseResult<ast::schema::Definition> {
        self.expect_name("type")?;
        let (name, _) = self.expect_name_value()?;
        let implements_interfaces = self.parse_implements_interfaces()?;
        let directives = self.parse_directives()?;
        let fields = self.parse_fields_definition()?;

        Ok(ast::schema::Definition::TypeExtension(
            ast::schema::TypeExtension::Object(
                ast::schema::ObjectTypeExtension {
                    position: self.span_to_pos(extend_span),
                    name,
                    implements_interfaces,
                    directives,
                    fields,
                },
            ),
        ))
    }

    /// Parses an interface type extension
    fn parse_interface_type_extension(
        &mut self,
        extend_span: Span,
    ) -> ParseResult<ast::schema::Definition> {
        self.expect_name("interface")?;
        let (name, _) = self.expect_name_value()?;
        let implements_interfaces = self.parse_implements_interfaces()?;
        let directives = self.parse_directives()?;
        let fields = self.parse_fields_definition()?;

        Ok(ast::schema::Definition::TypeExtension(
            ast::schema::TypeExtension::Interface(
                ast::schema::InterfaceTypeExtension {
                    position: self.span_to_pos(extend_span),
                    name,
                    implements_interfaces,
                    directives,
                    fields,
                },
            ),
        ))
    }

    /// Parses a union type extension
    fn parse_union_type_extension(
        &mut self,
        extend_span: Span,
    ) -> ParseResult<ast::schema::Definition> {
        self.expect_name("union")?;
        let (name, _) = self.expect_name_value()?;
        let directives = self.parse_directives()?;
        self.expect_punctuator("=")?;

        // Optional leading |
        let _ = self.skip_if_punctuator("|");

        let mut types = Vec::new();
        loop {
            let (type_name, _) = self.expect_name_value()?;
            types.push(type_name);

            if !self.skip_if_punctuator("|") {
                break;
            }
        }

        Ok(ast::schema::Definition::TypeExtension(
            ast::schema::TypeExtension::Union(
                ast::schema::UnionTypeExtension {
                    position: self.span_to_pos(extend_span),
                    name,
                    directives,
                    types,
                },
            ),
        ))
    }

    /// Parses an enum type extension
    fn parse_enum_type_extension(
        &mut self,
        extend_span: Span,
    ) -> ParseResult<ast::schema::Definition> {
        self.expect_name("enum")?;
        let (name, _) = self.expect_name_value()?;
        let directives = self.parse_directives()?;

        let open_brace_span = self.expect_punctuator("{")?;

        let mut values = Vec::new();
        while !self.tokens.check_punctuator("}") {
            if self.tokens.is_at_end() {
                return Err(self.unclosed_delimiter_error(
                    "{",
                    open_brace_span,
                ));
            }
            values.push(self.parse_enum_value_definition()?);
        }

        self.expect_punctuator("}")?;

        Ok(ast::schema::Definition::TypeExtension(
            ast::schema::TypeExtension::Enum(
                ast::schema::EnumTypeExtension {
                    position: self.span_to_pos(extend_span),
                    name,
                    directives,
                    values,
                },
            ),
        ))
    }

    /// Parses an input object type extension
    fn parse_input_object_type_extension(
        &mut self,
        extend_span: Span,
    ) -> ParseResult<ast::schema::Definition> {
        self.expect_name("input")?;
        let (name, _) = self.expect_name_value()?;
        let directives = self.parse_directives()?;

        let open_brace_span = self.expect_punctuator("{")?;

        let mut fields = Vec::new();
        while !self.tokens.check_punctuator("}") {
            if self.tokens.is_at_end() {
                return Err(self.unclosed_delimiter_error(
                    "{",
                    open_brace_span,
                ));
            }
            fields.push(self.parse_input_value_definition()?);
        }

        self.expect_punctuator("}")?;

        Ok(ast::schema::Definition::TypeExtension(
            ast::schema::TypeExtension::InputObject(
                ast::schema::InputObjectTypeExtension {
                    position: self.span_to_pos(extend_span),
                    name,
                    directives,
                    fields,
                },
            ),
        ))
    }

    /// Parses a scalar type extension
    fn parse_scalar_type_extension(
        &mut self,
        extend_span: Span,
    ) -> ParseResult<ast::schema::Definition> {
        self.expect_name("scalar")?;
        let (name, _) = self.expect_name_value()?;
        let directives = self.parse_directives()?;

        Ok(ast::schema::Definition::TypeExtension(
            ast::schema::TypeExtension::Scalar(
                ast::schema::ScalarTypeExtension {
                    position: self.span_to_pos(extend_span),
                    name,
                    directives,
                },
            ),
        ))
    }

    /// Parses a schema extension
    fn parse_schema_extension(
        &mut self,
        extend_span: Span,
    ) -> ParseResult<ast::schema::Definition> {
        self.expect_name("schema")?;
        let directives = self.parse_directives()?;

        let open_brace_span = self.expect_punctuator("{")?;

        let mut query = None;
        let mut mutation = None;
        let mut subscription = None;

        while !self.tokens.check_punctuator("}") {
            if self.tokens.is_at_end() {
                return Err(self.unclosed_delimiter_error(
                    "{",
                    open_brace_span,
                ));
            }

            let (operation_type, op_span) = self.expect_name_value()?;
            self.expect_punctuator(":")?;
            let (type_name, _) = self.expect_name_value()?;

            match operation_type.as_str() {
                "query" => query = Some(type_name),
                "mutation" => mutation = Some(type_name),
                "subscription" => subscription = Some(type_name),
                _ => {
                    return Err(GraphQLParseError::new(
                        format!(
                            "Invalid operation type '{}' in schema",
                            operation_type
                        ),
                        op_span,
                        GraphQLParseErrorKind::InvalidSyntax,
                    ))
                }
            }
        }

        self.expect_punctuator("}")?;

        // Schema extensions use SchemaDefinition variant
        Ok(ast::schema::Definition::SchemaDefinition(
            ast::schema::SchemaDefinition {
                position: self.span_to_pos(extend_span),
                directives,
                query,
                mutation,
                subscription,
            },
        ))
    }

    // ========== Error Recovery ==========

    /// Recovers to the next top-level definition
    fn recover_to_next_definition(&mut self) {
        while !self.tokens.is_at_end() {
            if self.tokens.check_name("type")
                || self.tokens.check_name("interface")
                || self.tokens.check_name("union")
                || self.tokens.check_name("enum")
                || self.tokens.check_name("scalar")
                || self.tokens.check_name("input")
                || self.tokens.check_name("directive")
                || self.tokens.check_name("schema")
                || self.tokens.check_name("extend")
            {
                break;
            }
            self.tokens.next();
        }
    }

    /// Recovers to the next field definition within a type
    fn recover_to_next_field(&mut self) {
        while !self.tokens.is_at_end() {
            if self.tokens.check_punctuator("}") {
                break;
            }
            // Look for pattern: Name ":" (likely field start)
            if self.tokens.peek().map(|(t, _)| matches!(t, GraphQLToken::Name(_))).unwrap_or(false)
                && self.tokens.peek_nth(1).map(|(t, _)| matches!(t, GraphQLToken::Punctuator(p) if p == ":")).unwrap_or(false)
            {
                break;
            }
            self.tokens.next();
        }
    }
}

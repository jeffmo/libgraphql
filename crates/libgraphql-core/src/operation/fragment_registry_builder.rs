use crate::ast;
use crate::file_reader;
use crate::loc;
use crate::operation::Fragment;
use crate::operation::FragmentBuilder;
use crate::operation::FragmentBuildError;
use crate::operation::FragmentRegistry;
use crate::operation::Selection;
use crate::operation::SelectionSet;
use crate::schema::Schema;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

type Result<T> = std::result::Result<T, Vec<FragmentRegistryBuildError>>;

/// Builder for constructing a [`FragmentRegistry`] with validation.
///
/// The `FragmentRegistryBuilder` allows you to incrementally add fragments
/// from multiple sources (files, strings, AST) and then build an immutable
/// [`FragmentRegistry`] with comprehensive validation including cycle
/// detection and reference checking.
///
/// # Example
///
/// ```
/// use libgraphql_core::schema::SchemaBuilder;
/// use libgraphql_core::operation::FragmentRegistryBuilder;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let schema = SchemaBuilder::from_str(
///     None,
///     "type Query { hello: String }"
/// )?
/// .build()?;
///
/// let mut builder = FragmentRegistryBuilder::new();
///
/// builder.add_from_document_str(
///     &schema,
///     "fragment UserFields on User { id name }",
///     None
/// ).unwrap();
///
/// builder.add_from_document_str(
///     &schema,
///     "fragment PostFields on Post { title body }",
///     None
/// ).unwrap();
///
/// let registry = builder.build().unwrap();
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct FragmentRegistryBuilder<'schema> {
    fragments: HashMap<String, Fragment<'schema>>,
}

impl<'schema> FragmentRegistryBuilder<'schema> {
    /// Create a new empty `FragmentRegistryBuilder`.
    pub fn new() -> Self {
        Self {
            fragments: HashMap::new(),
        }
    }

    /// Add a pre-built fragment to the registry.
    ///
    /// Returns an error if a fragment with the same name already exists.
    pub fn add_fragment(
        &mut self,
        fragment: Fragment<'schema>,
    ) -> std::result::Result<(), FragmentRegistryBuildError> {
        let name = fragment.name.clone();

        if let Some(existing) = self.fragments.get(&name) {
            return Err(FragmentRegistryBuildError::DuplicateFragmentDefinition {
                fragment_name: name,
                first_def_location: existing.def_location.clone(),
                second_def_location: fragment.def_location.clone(),
            });
        }

        self.fragments.insert(name, fragment);
        Ok(())
    }

    /// Parse fragments from an AST document and add them to the builder.
    ///
    /// This method follows the pattern of [`QueryBuilder::from_ast`](crate::operation::QueryBuilder::from_ast).
    ///
    /// Only fragment definitions in the document are processed; operation
    /// definitions are ignored.
    pub fn add_from_document_ast(
        &mut self,
        schema: &'schema Schema,
        ast: &ast::operation::Document,
        file_path: Option<&Path>,
    ) -> std::result::Result<(), Vec<FragmentBuildError>> {
        let mut errors = vec![];

        for def in &ast.definitions {
            if let ast::operation::Definition::Fragment(frag_def) = def {
                // Use empty registry for building - references validated later
                let temp_registry = FragmentRegistry::empty();

                match FragmentBuilder::from_ast(schema, temp_registry, frag_def, file_path)
                    .and_then(|builder| builder.build())
                {
                    Ok(fragment) => {
                        // Convert registry build error to fragment build error
                        if let Err(FragmentRegistryBuildError::DuplicateFragmentDefinition {
                            fragment_name,
                            first_def_location,
                            second_def_location,
                        }) = self.add_fragment(fragment) {
                            errors.push(FragmentBuildError::DuplicateFragmentDefinition {
                                fragment_name,
                                first_def_location,
                                second_def_location,
                            });
                        }
                    }
                    Err(e) => errors.push(e),
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }

    /// Parse fragments from a file and add them to the builder.
    ///
    /// This method follows the pattern of [`QueryBuilder::from_file`](crate::operation::QueryBuilder::from_file).
    pub fn add_from_document_file(
        &mut self,
        schema: &'schema Schema,
        file_path: impl AsRef<Path>,
    ) -> std::result::Result<(), Vec<FragmentBuildError>> {
        let file_path = file_path.as_ref();
        let file_content = file_reader::read_content(file_path)
            .map_err(|e| vec![FragmentBuildError::FileReadError(Box::new(e))])?;

        self.add_from_document_str(schema, file_content, Some(file_path))
    }

    /// Parse fragments from a string and add them to the builder.
    ///
    /// This method follows the pattern of [`QueryBuilder::from_str`](crate::operation::QueryBuilder::from_str).
    pub fn add_from_document_str(
        &mut self,
        schema: &'schema Schema,
        content: impl AsRef<str>,
        file_path: Option<&Path>,
    ) -> std::result::Result<(), Vec<FragmentBuildError>> {
        let ast_doc = ast::operation::parse(content.as_ref())
            .map_err(|e| vec![FragmentBuildError::ParseError(Arc::new(e))])?;

        self.add_from_document_ast(schema, &ast_doc, file_path)
    }

    /// Build the immutable [`FragmentRegistry`] with comprehensive validation.
    ///
    /// This method performs the following validations:
    /// - Detects cycles in fragment spreads
    /// - Deduplicates phase-shifted cycles (e.g., A→B→C→A is the same as B→C→A→B)
    /// - Validates that all fragment references exist
    ///
    /// If any validation errors are found, returns all errors at once rather
    /// than failing on the first error.
    pub fn build(self) -> Result<FragmentRegistry<'schema>> {
        let mut errors = Vec::new();

        // Collect all cycle errors
        errors.extend(self.validate_no_cycles());

        // Collect all undefined reference errors
        errors.extend(self.validate_fragment_references());

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(FragmentRegistry {
            fragments: self.fragments,
        })
    }

    /// Validate that no cycles exist in fragment spreads.
    ///
    /// Uses DFS traversal with cycle normalization to detect and deduplicate
    /// cycles. Phase-shifted cycles (rotations of the same cycle) are
    /// deduplicated.
    fn validate_no_cycles(&self) -> Vec<FragmentRegistryBuildError> {
        let mut all_cycles = Vec::new();
        let mut seen_normalized_cycles = HashSet::new();

        for fragment_name in self.fragments.keys() {
            let mut path = Vec::new();
            let mut visiting = HashSet::new();

            self.check_fragment_cycles(
                fragment_name,
                &mut path,
                &mut visiting,
                &mut all_cycles,
                &mut seen_normalized_cycles,
            );
        }

        all_cycles
    }

    fn check_fragment_cycles(
        &self,
        fragment_name: &str,
        path: &mut Vec<String>,
        visiting: &mut HashSet<String>,
        errors: &mut Vec<FragmentRegistryBuildError>,
        seen_normalized: &mut HashSet<Vec<String>>,
    ) {
        // Cycle detected
        if visiting.contains(fragment_name) {
            path.push(fragment_name.to_string());

            // Normalize the cycle to check for duplicates
            let normalized = Self::normalize_cycle(path);

            // Only add if we haven't seen this cycle before (in any phase)
            if !seen_normalized.contains(&normalized) {
                seen_normalized.insert(normalized);
                errors.push(FragmentRegistryBuildError::FragmentCycleDetected {
                    cycle_path: path.clone(),
                });
            }

            path.pop();
            return;
        }

        // Fragment doesn't exist - will be caught by reference validation
        let Some(fragment) = self.fragments.get(fragment_name) else {
            return;
        };

        path.push(fragment_name.to_string());
        visiting.insert(fragment_name.to_string());

        // Recursively check all fragment spreads
        self.check_selection_set_cycles(
            &fragment.selection_set,
            path,
            visiting,
            errors,
            seen_normalized,
        );

        path.pop();
        visiting.remove(fragment_name);
    }

    fn check_selection_set_cycles(
        &self,
        selection_set: &SelectionSet<'schema>,
        path: &mut Vec<String>,
        visiting: &mut HashSet<String>,
        errors: &mut Vec<FragmentRegistryBuildError>,
        seen_normalized: &mut HashSet<Vec<String>>,
    ) {
        for selection in &selection_set.selections {
            match selection {
                Selection::FragmentSpread(spread) => {
                    self.check_fragment_cycles(
                        spread.fragment_name(),
                        path,
                        visiting,
                        errors,
                        seen_normalized,
                    );
                }
                Selection::InlineFragment(inline) => {
                    self.check_selection_set_cycles(
                        inline.selection_set(),
                        path,
                        visiting,
                        errors,
                        seen_normalized,
                    );
                }
                Selection::Field(field) => {
                    if let Some(nested_set) = field.selection_set() {
                        self.check_selection_set_cycles(
                            nested_set,
                            path,
                            visiting,
                            errors,
                            seen_normalized,
                        );
                    }
                }
            }
        }
    }

    /// Validate that all fragment references point to existing fragments.
    fn validate_fragment_references(&self) -> Vec<FragmentRegistryBuildError> {
        let mut errors = Vec::new();

        for (fragment_name, fragment) in &self.fragments {
            self.check_fragment_refs_in_selection_set(
                fragment_name,
                &fragment.selection_set,
                &mut errors,
            );
        }

        errors
    }

    fn check_fragment_refs_in_selection_set(
        &self,
        parent_fragment: &str,
        selection_set: &SelectionSet<'schema>,
        errors: &mut Vec<FragmentRegistryBuildError>,
    ) {
        for selection in &selection_set.selections {
            match selection {
                Selection::FragmentSpread(spread) => {
                    let ref_name = spread.fragment_name();
                    if !self.fragments.contains_key(ref_name) {
                        errors.push(FragmentRegistryBuildError::UndefinedFragmentReference {
                            fragment_name: parent_fragment.to_string(),
                            undefined_fragment: ref_name.to_string(),
                            reference_location: spread.def_location.clone(),
                        });
                    }
                }
                Selection::InlineFragment(inline) => {
                    self.check_fragment_refs_in_selection_set(
                        parent_fragment,
                        inline.selection_set(),
                        errors,
                    );
                }
                Selection::Field(field) => {
                    if let Some(nested_set) = field.selection_set() {
                        self.check_fragment_refs_in_selection_set(
                            parent_fragment,
                            nested_set,
                            errors,
                        );
                    }
                }
            }
        }
    }

    /// Normalize a cycle to canonical form for deduplication.
    ///
    /// Cycles that are rotations of each other are considered identical.
    /// For example, `[A, B, C, A]`, `[B, C, A, B]`, and `[C, A, B, C]` are
    /// all the same cycle.
    ///
    /// Normalization rotates the cycle to start with the lexicographically
    /// smallest fragment name.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// normalize_cycle(&["B", "C", "A", "B"]) // => ["A", "B", "C", "A"]
    /// normalize_cycle(&["C", "A", "B", "C"]) // => ["A", "B", "C", "A"]
    /// ```
    fn normalize_cycle(cycle: &[String]) -> Vec<String> {
        if cycle.is_empty() {
            return Vec::new();
        }

        // Remove the duplicate last element: [A, B, C, A] → [A, B, C]
        let cycle_without_repeat = &cycle[..cycle.len() - 1];

        // Find the position of the lexicographically smallest fragment
        let min_idx = cycle_without_repeat
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        // Rotate to start from the minimum
        let mut normalized = Vec::new();
        normalized.extend_from_slice(&cycle_without_repeat[min_idx..]);
        normalized.extend_from_slice(&cycle_without_repeat[..min_idx]);

        // Add back the duplicate last element
        normalized.push(normalized[0].clone());

        normalized
    }
}

impl<'schema> Default for FragmentRegistryBuilder<'schema> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Error)]
pub enum FragmentRegistryBuildError {
    #[error("Duplicate fragment definition: '{fragment_name}'")]
    DuplicateFragmentDefinition {
        fragment_name: String,
        first_def_location: loc::SourceLocation,
        second_def_location: loc::SourceLocation,
    },

    #[error("Fragment cycle detected: {}", format_cycle_path(.cycle_path))]
    FragmentCycleDetected { cycle_path: Vec<String> },

    #[error("Fragment '{fragment_name}' references undefined fragment '{undefined_fragment}'")]
    UndefinedFragmentReference {
        fragment_name: String,
        undefined_fragment: String,
        reference_location: loc::SourceLocation,
    },
}

fn format_cycle_path(cycle: &[String]) -> String {
    cycle.join(" → ")
}

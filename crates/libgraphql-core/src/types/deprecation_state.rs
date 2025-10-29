use crate::DirectiveAnnotation;

pub enum DeprecationState<'a> {
    Deprecated(&'a str),
    NotDeprecated,
}
impl<'a> DeprecationState<'a> {
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Self::Deprecated(_))
    }
}

impl<'a> std::convert::From<&'a Vec<DirectiveAnnotation>> for DeprecationState<'a> {
    fn from(value: &'a Vec<DirectiveAnnotation>) -> DeprecationState<'a> {
        let directive_annot = value.iter().find(|directive_annot| {
            directive_annot.directive_type_name() == "deprecated"
        });
        if let Some(directive_annot) = directive_annot {
            let reason =
                directive_annot.args
                    .get("reason")
                    .expect("no `reason` argument found")
                    .as_str()
                    .expect("`reason` argument found as non-string type");
            DeprecationState::Deprecated(reason)
        } else {
            DeprecationState::NotDeprecated
        }
    }
}

use crate::{operation::FragmentSet, schema::Schema};

pub struct DocumentBuilder<'schema, 'fragset> {
    schema: Option<&'schema Schema>,
    fragset: Option<&'fragset FragmentSet<'schema>>,
}

// TODO

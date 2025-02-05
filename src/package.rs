use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Package(HashMap<String, String>);

impl Package {
    pub fn new(fields: HashMap<String, String>) -> Self {
        Self(fields)
    }
}

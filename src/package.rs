use derive_more::Deref;
use indexmap::IndexMap;
use tabled::Table;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deref)]
pub struct Package(IndexMap<String, String>);

impl Package {
    pub fn new(fields: IndexMap<String, String>) -> Self {
        Self(fields)
    }

    pub fn list_into_tab(packages: Vec<Self>) -> Table {
        let mut t = tabled::builder::Builder::default();
        let Some(first) = packages.first() else {
            return Table::default();
        };
        t.push_record(first.keys());
        for package in packages {
            t.push_record(package.0.into_values());
        }
        t.build()
    }

    pub fn into_tab(self) -> Table {
        let mut t = tabled::builder::Builder::with_capacity(self.len(), 2);
        for (key, value) in self.0 {
            t.push_record([key, value]);
        }
        t.build()
    }
}

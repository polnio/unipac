use abi_stable::export_root_module;
use unipac_shared::{Plugin, SharedPlugin};

#[export_root_module]
pub fn get_plugin() -> SharedPlugin {
    Plugin {
        name: "Unipac Dummy Plugin".into(),
    }
    .into()
}

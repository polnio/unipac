use abi_stable::{
    declare_root_module_statics, library::RootModule, package_version_strings,
    prefix_type::PrefixTypeTrait as _, sabi_types::VersionStrings, std_types::RStr, StableAbi,
};

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = SharedPlugin)))]
pub struct Plugin {
    pub name: RStr<'static>,
}

impl RootModule for SharedPlugin {
    const BASE_NAME: &'static str = "unipac_plugin";
    const NAME: &'static str = Self::BASE_NAME;
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
    declare_root_module_statics! {SharedPlugin}
}

impl From<Plugin> for SharedPlugin {
    fn from(value: Plugin) -> Self {
        value.leak_into_prefix()
    }
}

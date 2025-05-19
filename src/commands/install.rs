use anyhow::Context as _;
use dialoguer::Select;
use itertools::Itertools;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct InstallPackage<'a> {
    plugin_id: &'a str,
    plugin_name: &'a str,
    package_id: &'a str,
    package_version: &'a str,
}

pub fn install(pname: &'static str) {
    let handles = super::fetch(move |p| p.pre_install(pname).context("Failed to check packages"));

    let (handles, errors): (Vec<_>, Vec<_>) = handles.into_iter().partition_result();
    for err in errors {
        eprintln!("{}", err);
    }

    let packages = handles
        .iter()
        .flat_map(|result| {
            let super::PluginResult { data, id, name, .. } = result;
            data.into_iter().filter_map(move |package| {
                Some(InstallPackage {
                    plugin_id: &id,
                    plugin_name: &name,
                    package_id: package.get("Id")?,
                    package_version: package.get("Version")?,
                })
            })
        })
        .collect_vec();

    let options = packages
        .iter()
        .map(|package| {
            format!(
                "{}: {} {}",
                package.plugin_name, package.package_id, package.package_version
            )
        })
        .collect_vec();

    let selected = Select::new()
        .with_prompt("Select a package to install")
        .items(&options)
        .default(0)
        .interact_opt()
        .unwrap();

    let Some(selected) = selected else {
        println!("No package selected");
        return;
    };

    let selected_package = packages.get(selected).unwrap();

    let result = super::fetch_id(selected_package.plugin_id.to_owned(), move |p| {
        p.install(selected_package.package_id)
            .context("Failed to install package")
    });
    if let Err(err) = result {
        eprintln!("Error: {:#}", err);
    }
}

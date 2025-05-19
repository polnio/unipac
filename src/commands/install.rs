use anyhow::Context as _;
use dialoguer::Select;
use itertools::Itertools;

use super::Args;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct InstallPackage<'a> {
    plugin_id: &'a str,
    plugin_name: &'a str,
    plugin_color: &'a str,
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
            let super::PluginResult {
                data,
                id,
                name,
                color,
                ..
            } = result;
            data.into_iter().filter_map(move |package| {
                Some(InstallPackage {
                    plugin_id: &id,
                    plugin_name: &name,
                    plugin_color: &color,
                    package_id: package.get("Id")?,
                    package_version: package.get("Version")?,
                })
            })
        })
        .collect_vec();

    let args = Args::get();
    let options = packages
        .iter()
        .map(|package| {
            let row = format!(
                "{}: {} {}",
                package.plugin_name, package.package_id, package.package_version
            );
            let style = if args.colors {
                console::Style::from_dotted_str(&package.plugin_color)
            } else {
                console::Style::default()
            };
            style.apply_to(row)
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
        if args.colors {
            let error = console::style("Error").red();
            let err = console::Style::from_dotted_str(selected_package.plugin_color).apply_to(err);
            eprintln!("{}: {:#}", error, err);
        } else {
            eprintln!("Error: {:#}", err);
        };
    }
}

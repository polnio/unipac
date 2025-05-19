use crate::{Package, Result};
use anyhow::{bail, Context as _};
use std::io::{BufRead as _, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Response {
    Progress(u8),
    Response(String),
    Package(Package),
    End,
    Error(String),
}
impl Response {
    pub fn new_progress_from_str(s: &str) -> Self {
        match s.parse() {
            Ok(progress) => Response::Progress(progress),
            Err(err) => Response::Error(format!("Failed to parse progress: {}", err)),
        }
    }
}
impl From<Result<Package>> for Response {
    fn from(result: Result<Package>) -> Self {
        match result {
            Ok(package) => Response::Package(package),
            Err(err) => Response::Error(format!("Failed to parse package: {}", err)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Progress(u8),
    End,
}

pub struct PluginBuilderPath(String);
pub struct PluginBuilderPathEmpty;
#[derive(Debug, Clone)]
pub struct PluginBuilder<Path> {
    path: Path,
    event_sender: Option<mpsc::Sender<Event>>,
}
impl Default for PluginBuilder<PluginBuilderPathEmpty> {
    fn default() -> Self {
        Self {
            path: PluginBuilderPathEmpty,
            event_sender: None,
        }
    }
}
impl PluginBuilder<PluginBuilderPathEmpty> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl<Path> PluginBuilder<Path> {
    pub fn path(self, path: String) -> PluginBuilder<PluginBuilderPath> {
        PluginBuilder {
            path: PluginBuilderPath(path),
            event_sender: self.event_sender,
        }
    }
    pub fn event_sender(self, event_sender: mpsc::Sender<Event>) -> Self {
        Self {
            event_sender: Some(event_sender),
            ..self
        }
    }
}
impl PluginBuilder<PluginBuilderPath> {
    pub fn build(self) -> Plugin {
        Plugin::new(self.path.0, self.event_sender)
    }
}

#[derive(Debug)]
pub struct Plugin {
    pub path: String,
    response_sender: mpsc::Sender<Response>,
    response_receiver: mpsc::Receiver<Response>,
    event_sender: Option<mpsc::Sender<Event>>,
}

macro_rules! impl_plugin_inner {
    ($vis:vis fn $name:ident ($($field:ident : $type:ty),*) -> $return:ty, $getter:ident) => {
        paste::paste! {
            impl Plugin {
                $vis fn $name(&self, $($field: $type),*) -> Result<$return> {
                    let handle = self.start_subcommand(stringify!([<unipac_ $name>]), &[$($field),*]);
                    let packages = self.$getter()?;
                    handle.join().unwrap()?;
                    Ok(packages)
                }
            }
        }
    };
}
macro_rules! impl_plugin {
    ($vis:vis fn $name:ident ($($field:ident : $type:ty),*)) => {
        impl_plugin_inner!($vis fn $name ($($field : $type),*) -> (), get_nothing);
    };
    ($vis:vis fn $name:ident ($($field:ident : $type:ty),*) -> Vec<Package>) => {
        impl_plugin_inner!($vis fn $name ($($field : $type),*) -> Vec<Package>, get_packages);
    };
    ($vis:vis fn $name:ident ($($field:ident : $type:ty),*) -> Option<Package>) => {
        impl_plugin_inner!($vis fn $name ($($field : $type),*) -> Option<Package>, get_package);
    };
    ($vis:vis fn $name:ident ($($field:ident : $type:ty),*) -> String) => {
        impl_plugin_inner!($vis fn $name ($($field : $type),*) -> String, get_response);
    };
}

fn send_opt<T>(sender: Option<&mpsc::Sender<T>>, value: T) {
    if let Some(sender) = sender {
        let _ = sender.send(value);
    }
}

impl_plugin!(pub fn get_id() -> String);
impl_plugin!(pub fn get_name() -> String);
impl_plugin!(pub fn get_color() -> String);
impl_plugin!(pub fn list_packages() -> Vec<Package>);
impl_plugin!(pub fn search(query: &str) -> Vec<Package>);
impl_plugin!(pub fn info(pname: &str) -> Option<Package>);
impl_plugin!(pub fn pre_install(pname: &str) -> Vec<Package>);
impl_plugin!(pub fn install(pname: &str));
impl_plugin!(pub fn remove(pname: &str));
impl Plugin {
    pub fn builder() -> PluginBuilder<PluginBuilderPathEmpty> {
        PluginBuilder::new()
    }
    fn new(path: String, event_sender: Option<mpsc::Sender<Event>>) -> Self {
        let (response_sender, response_receiver) = mpsc::channel();
        Self {
            path,
            response_sender,
            response_receiver: response_receiver.into(),
            event_sender,
        }
    }
    fn get_response(&self) -> Result<String> {
        while let Ok(response) = self.response_receiver.recv() {
            match response {
                Response::Response(response) => return Ok(response),
                Response::Progress(progress) => {
                    send_opt(self.event_sender.as_ref(), Event::Progress(progress))
                }
                Response::End => {
                    send_opt(self.event_sender.as_ref(), Event::End);
                    bail!("Ended without response")
                }
                Response::Error(error) => {
                    send_opt(self.event_sender.as_ref(), Event::End);
                    bail!(error)
                }
                _ => {}
            }
        }
        bail!("Ended without response")
    }
    fn get_packages(&self) -> Result<Vec<Package>> {
        let mut packages = Vec::new();
        while let Ok(response) = self.response_receiver.recv() {
            match response {
                Response::Package(package) => packages.push(package),
                Response::Progress(progress) => {
                    send_opt(self.event_sender.as_ref(), Event::Progress(progress));
                    if progress == 100 {
                        break;
                    }
                }
                Response::End => {
                    send_opt(self.event_sender.as_ref(), Event::End);
                    break;
                }
                Response::Error(error) => {
                    send_opt(self.event_sender.as_ref(), Event::End);
                    bail!(error)
                }
                _ => {}
            }
        }
        Ok(packages)
    }
    fn get_package(&self) -> Result<Option<Package>> {
        self.get_packages().map(|ps| ps.into_iter().nth(0))
    }
    fn get_nothing(&self) -> Result<()> {
        while let Ok(response) = self.response_receiver.recv() {
            match response {
                Response::Progress(progress) => {
                    send_opt(self.event_sender.as_ref(), Event::Progress(progress));
                    if progress == 100 {
                        break;
                    }
                }
                Response::End => {
                    send_opt(self.event_sender.as_ref(), Event::End);
                }
                Response::Error(error) => {
                    send_opt(self.event_sender.as_ref(), Event::End);
                    bail!(error)
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn start_subcommand(&self, command: &'static str, args: &[&str]) -> JoinHandle<Result<()>> {
        let response_sender = self.response_sender.clone();
        let path = self.path.clone();
        let args = args.join(" ");
        std::thread::spawn(move || {
            let mut cmd = Command::new("bash")
                .arg("-c")
                .arg(format!("{} {} {}", path, command, args))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .context("Failed to run plugin")?;
            let stdout = cmd.stdout.as_mut().context("Failed to get stdout")?;
            let lines = BufReader::new(stdout).lines();
            let mut fields: Option<Vec<String>> = None;
            for line in lines {
                let Ok(line) = line else {
                    continue;
                };
                let Some((channel, data_str)) = line.split_once(' ') else {
                    continue;
                };
                match channel {
                    "Package" => {
                        if let Some(fields) = &fields {
                            let data = fields
                                .iter()
                                .zip(data_str.split(','))
                                .map(|(field, value)| (field.to_owned(), value.to_owned()))
                                .collect();
                            let package = Package::new(data);
                            let _ = response_sender.send(Response::from(Ok(package)));
                        } else {
                            fields = Some(data_str.split(',').map(str::to_owned).collect());
                        }
                    }
                    "Progress" => {
                        let _ = response_sender.send(Response::new_progress_from_str(data_str));
                    }
                    "Response" => {
                        let _ = response_sender.send(Response::Response(data_str.to_owned()));
                    }
                    "End" => {
                        let _ = response_sender.send(Response::End);
                        break;
                    }
                    "Error" => {
                        let _ = response_sender.send(Response::Error(data_str.to_owned()));
                    }
                    _ => continue,
                }
            }

            let status = cmd.wait().expect("Failed to wait for plugin");
            if !status.success() {
                let stderr = cmd.stderr.as_mut().context("Failed to get stderr")?;
                let mut buffer = Vec::new();
                match stderr.read_to_end(&mut buffer) {
                    Ok(n) if n != 0 => {
                        let message = String::from_utf8_lossy(&buffer);
                        bail!(
                            "Plugin failed with status {} and error: {}",
                            status,
                            message
                        );
                    }
                    _ => {
                        bail!("Plugin failed with status {}", status);
                    }
                }
            }
            Ok(())
        })
    }
}

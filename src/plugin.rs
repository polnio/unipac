use crate::{Package, Result};
use anyhow::{bail, Context as _};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

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

pub struct PluginBuilderPath(String);
pub struct PluginBuilderPathEmpty;
#[derive(Debug, Clone)]
pub struct PluginBuilder<Path> {
    path: Path,
    progress_sender: Option<mpsc::Sender<u8>>,
    end_sender: Option<mpsc::Sender<()>>,
}
impl Default for PluginBuilder<PluginBuilderPathEmpty> {
    fn default() -> Self {
        Self {
            path: PluginBuilderPathEmpty,
            progress_sender: None,
            end_sender: None,
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
            progress_sender: self.progress_sender,
            end_sender: self.end_sender,
        }
    }
    pub fn progress_sender(self, progress_sender: mpsc::Sender<u8>) -> Self {
        Self {
            progress_sender: Some(progress_sender),
            ..self
        }
    }
    pub fn end_sender(self, end_sender: mpsc::Sender<()>) -> Self {
        Self {
            end_sender: Some(end_sender),
            ..self
        }
    }
}
impl PluginBuilder<PluginBuilderPath> {
    pub fn build(self) -> Plugin {
        Plugin::new(self.path.0, self.progress_sender, self.end_sender)
    }
}

#[derive(Debug)]
pub struct Plugin {
    pub path: String,
    response_sender: mpsc::Sender<Response>,
    response_receiver: Mutex<mpsc::Receiver<Response>>,
    progress_sender: Option<mpsc::Sender<u8>>,
    end_sender: Option<mpsc::Sender<()>>,
}

macro_rules! impl_plugin_inner {
    ($vis:vis async fn $name:ident ($($field:ident : $type:ty),*) -> $return:ty, $getter:ident) => {
        impl Plugin {
            $vis async fn $name(&self, $($field: $type),*) -> Result<$return> {
                let handle = self.start_subcommand(stringify!($name), vec![$($field),*]);
                let packages = self.$getter();
                let (_, packages) = tokio::try_join!(handle, packages)?;
                Ok(packages)
            }
        }
    };
}
macro_rules! impl_plugin {
    ($vis:vis async fn $name:ident ($($field:ident : $type:ty),*) -> Vec<Package>) => {
        impl_plugin_inner!($vis async fn $name ($($field : $type),*) -> Vec<Package>, get_packages);
    };
    ($vis:vis async fn $name:ident ($($field:ident : $type:ty),*) -> String) => {
        impl_plugin_inner!($vis async fn $name ($($field : $type),*) -> String, get_response);
    };
}

async fn send_opt<T>(sender: Option<&mpsc::Sender<T>>, value: T) {
    if let Some(sender) = sender {
        let _ = sender.send(value).await;
    }
}

impl_plugin!(pub async fn get_id() -> String);
impl_plugin!(pub async fn get_name() -> String);
impl_plugin!(pub async fn list_packages() -> Vec<Package>);
impl Plugin {
    pub fn builder() -> PluginBuilder<PluginBuilderPathEmpty> {
        PluginBuilder::new()
    }
    fn new(
        path: String,
        progress_sender: Option<mpsc::Sender<u8>>,
        end_sender: Option<mpsc::Sender<()>>,
    ) -> Self {
        let (response_sender, response_receiver) = mpsc::channel(100);
        Self {
            path,
            response_sender,
            response_receiver: response_receiver.into(),
            progress_sender,
            end_sender,
        }
    }
    async fn get_response(&self) -> Result<String> {
        let mut response_receiver = self.response_receiver.lock().await;
        loop {
            let response = response_receiver.recv().await.unwrap();
            match response {
                Response::Response(response) => return Ok(response),
                Response::Progress(progress) => {
                    send_opt(self.progress_sender.as_ref(), progress).await
                }
                Response::End => {
                    send_opt(self.end_sender.as_ref(), ()).await;
                    bail!("Ended without response")
                }
                Response::Error(error) => {
                    send_opt(self.end_sender.as_ref(), ()).await;
                    bail!(error)
                }
                _ => {}
            }
        }
    }
    async fn get_packages(&self) -> Result<Vec<Package>> {
        let mut packages = Vec::new();
        let mut response_receiver = self.response_receiver.lock().await;
        loop {
            match response_receiver.recv().await.unwrap() {
                Response::Package(package) => packages.push(package),
                Response::Progress(progress) => {
                    send_opt(self.progress_sender.as_ref(), progress).await;
                    if progress == 100 {
                        break;
                    }
                }
                Response::End => {
                    send_opt(self.end_sender.as_ref(), ()).await;
                    break;
                }
                Response::Error(error) => {
                    send_opt(self.end_sender.as_ref(), ()).await;
                    bail!(error)
                }
                _ => {}
            }
        }
        Ok(packages)
    }
    async fn start_subcommand(&self, command: &'static str, args: Vec<String>) -> Result<()> {
        let response_sender = self.response_sender.clone();
        let path = self.path.clone();
        let mut cmd = Command::new("bash")
            .arg("-c")
            .arg(format!("{} {} {}", path, command, args.join(" ")))
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to run plugin")?;
        let stdout = cmd.stdout.as_mut().context("Failed to get stdout")?;
        let mut lines = BufReader::new(stdout).lines();
        let mut fields: Option<Vec<String>> = None;
        while let Some(line) = lines.next_line().await? {
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
                        let _ = response_sender.send(Response::from(Ok(package))).await;
                    } else {
                        fields = Some(data_str.split(',').map(str::to_owned).collect());
                    }
                }
                "Progress" => {
                    let _ = response_sender
                        .send(Response::new_progress_from_str(data_str))
                        .await;
                }
                "Response" => {
                    let _ = response_sender
                        .send(Response::Response(data_str.to_owned()))
                        .await;
                }
                "End" => {
                    let _ = response_sender.send(Response::End).await;
                    break;
                }
                "Error" => {
                    let _ = response_sender
                        .send(Response::Error(data_str.to_owned()))
                        .await;
                }
                _ => continue,
            }
        }

        cmd.wait().await.expect("Failed to wait for plugin");
        Ok(())
    }
}

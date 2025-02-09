use crate::{Package, Result};
use anyhow::{bail, Context as _};
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

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

#[derive(Debug)]
pub struct Plugin {
    pub path: String,
    response_sender: mpsc::Sender<Response>,
    // response_receiver: mpsc::Receiver<Response>,
    response_receiver: tokio::sync::Mutex<mpsc::Receiver<Response>>,
    progress_sender: mpsc::Sender<u8>,
    end_sender: mpsc::Sender<()>,
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

impl_plugin!(pub async fn get_id() -> String);
impl_plugin!(pub async fn get_name() -> String);
impl_plugin!(pub async fn list_packages() -> Vec<Package>);
impl Plugin {
    pub fn new(
        path: String,
        progress_sender: mpsc::Sender<u8>,
        end_sender: mpsc::Sender<()>,
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
                Response::Progress(progress) => self.progress_sender.send(progress).await.unwrap(),
                Response::End => {
                    self.end_sender.send(()).await.unwrap();
                    bail!("Ended without response")
                }
                Response::Error(error) => {
                    self.end_sender.send(()).await.unwrap();
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
                    self.progress_sender.send(progress).await.unwrap();
                    if progress == 100 {
                        break;
                    }
                }
                Response::End => {
                    self.end_sender.send(()).await.unwrap();
                    break;
                }
                Response::Error(error) => {
                    self.end_sender.send(()).await.unwrap();
                    bail!(error)
                }
                _ => {}
            }
        }
        Ok(packages)
    }
    async fn start_subcommand(&self, command: &'static str, args: Vec<String>) -> Result<()> {
        // while self.response_receiver.try_recv().is_ok() {}
        let response_sender = self.response_sender.clone();
        let path = self.path.clone();
        let mut cmd = Command::new("bash")
            .arg("-c")
            .arg(format!("{} {} {}", path, command, args.join(" ")))
            .stdout(std::process::Stdio::piped())
            .spawn()
            .context("Failed to run plugin")?;
        let stdout = cmd.stdout.as_mut().context("Failed to get stdout")?;
        let mut lines = BufReader::new(stdout).lines();
        let mut fields: Option<Vec<String>> = None;
        while let Some(line) = lines.next_line().await? {
            // let line = line.expect("Failed to read line");
            let Some((channel, data_str)) = line.split_once(' ') else {
                continue;
            };
            match channel {
                "Package" => {
                    // let _ = response_sender.send(Response::from(Package::from_json(data_str)));
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

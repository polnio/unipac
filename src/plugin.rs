use crate::{Package, Result};
use anyhow::{bail, Context as _};
use std::io::{BufRead as _, BufReader};
use std::process::Command;
use std::sync::mpsc;
use std::thread::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Response {
    Progress(u8),
    Response(String),
    Package(Package),
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
    response_receiver: mpsc::Receiver<Response>,
    progress_sender: mpsc::Sender<u8>,
}

macro_rules! impl_plugin_inner {
    ($vis:vis fn $name:ident ($($field:ident : $type:ty),*) -> $return:ty, $getter:ident) => {
        impl Plugin {
            $vis fn $name(&self, $($field: $type),*) -> Result<$return> {
                let handle = self.start_subcommand(stringify!($name), vec![$($field),*]);
                let packages = self.$getter()?;
                let _ = handle.join();
                Ok(packages)
            }
        }
    };
}
macro_rules! impl_plugin {
    ($vis:vis fn $name:ident ($($field:ident : $type:ty),*) -> Vec<Package>) => {
        impl_plugin_inner!($vis fn $name ($($field : $type),*) -> Vec<Package>, get_packages);
    };
    ($vis:vis fn $name:ident ($($field:ident : $type:ty),*) -> String) => {
        impl_plugin_inner!($vis fn $name ($($field : $type),*) -> String, get_response);
    };
}

impl_plugin!(pub fn get_id() -> String);
impl_plugin!(pub fn get_name() -> String);
impl_plugin!(pub fn list_packages() -> Vec<Package>);
impl Plugin {
    pub fn new(path: String, progress_sender: mpsc::Sender<u8>) -> Self {
        let (response_sender, response_receiver) = mpsc::channel();
        Self {
            path: path.into(),
            response_sender,
            response_receiver,
            progress_sender,
        }
    }
    fn get_response(&self) -> Result<String> {
        loop {
            let response = self.response_receiver.recv().unwrap();
            match response {
                Response::Response(response) => return Ok(response),
                Response::Progress(progress) => self.progress_sender.send(progress).unwrap(),

                Response::Error(error) => bail!(error),
                _ => continue,
            }
        }
    }
    fn get_packages(&self) -> Result<Vec<Package>> {
        let mut packages = Vec::new();
        loop {
            match self.response_receiver.recv().unwrap() {
                Response::Package(package) => packages.push(package),
                Response::Progress(progress) => {
                    self.progress_sender.send(progress).unwrap();
                    if progress == 100 {
                        break;
                    }
                }
                Response::Error(error) => bail!(error),
                _ => continue,
            }
        }
        Ok(packages)
    }
    fn start_subcommand(&self, command: &'static str, args: Vec<String>) -> JoinHandle<Result<()>> {
        // while self.response_receiver.try_recv().is_ok() {}
        let response_sender = self.response_sender.clone();
        let path = self.path.clone();
        std::thread::spawn(move || {
            let mut cmd = Command::new("bash")
                .arg("-c")
                .arg(format!("{} {} {}", path, command, args.join(" ")))
                .stdout(std::process::Stdio::piped())
                .spawn()
                .context("Failed to run plugin")?;
            let stdout = cmd.stdout.as_mut().context("Failed to get stdout")?;
            let lines = BufReader::new(stdout).lines();
            let mut fields: Option<Vec<String>> = None;
            for line in lines {
                // let line = line.expect("Failed to read line");
                let Ok(line) = line else {
                    continue;
                };
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
                    "Error" => {
                        let _ = response_sender.send(Response::Error(data_str.to_owned()));
                    }
                    _ => continue,
                }
            }

            cmd.wait().expect("Failed to wait for plugin");
            Ok(())
        })
    }
}

use std::fs::read_to_string;
use std::io::ErrorKind;
use std::path;

#[derive(Debug)]
#[derive(serde::Deserialize)]
pub struct Config {
    pub global: Global,
    pub actions: Vec<Action>,
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
#[derive(serde::Deserialize)]
pub struct Global {
    pub port: u16,
}

#[derive(Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Action {
    pub name: String,
    pub target: String,
    pub method: Option<String>,
}

#[derive(Debug)]
#[derive(serde::Deserialize)]
pub struct Rule {
    pub name: String,
    pub trigger: String,
    pub actions: Vec<String>,
}

pub fn read_config(cfg_path: &str) -> Result<Config,std::io::Error>{
    println!("Read config \"{}\"", cfg_path);
    let path = path::Path::new(cfg_path);
    if path.is_file() {
        return read_config_file(path);
    }
    return Err(std::io::Error::new(ErrorKind::Unsupported, "not a file"));
}

fn read_config_file(path: &path::Path) -> Result<Config, std::io::Error>{
    let file_data = read_to_string(path)?;
    let file_str = file_data.as_str();

    toml::from_str(file_str).map_err(|e| std::io::Error::new(ErrorKind::InvalidData,e.to_string()))
}


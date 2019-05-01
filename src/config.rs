use {
    handlebars::Handlebars,
    serde_json::value::Map,
    std::{
        io::{Error, ErrorKind},
        fs::File,
        path::Path,
        process::Command,
    },
};

#[derive(Deserialize, Clone)]
pub struct Service {
    pub service_name: String,
    pub parameters: Vec<Parameter>
}

#[derive(Deserialize, Clone)]
pub struct Parameter {
    pub ps_path: String,
    pub name: String,
    pub value: serde_json::Value
}

#[derive(Deserialize)]
pub struct Config {
    pub settings: Vec<Setting>
}

#[derive(Deserialize)]
pub struct Setting {
    pub service_name: String,
    pub keys: Vec<String>,
    pub template: String,
    pub rendered: String,
    pub reload_cmd: String,
}

pub fn render_config(services: &Vec<Service>, config: &Config) -> Result<(), std::io::Error> {
    let handlebars = Handlebars::new();
    let mut data = Map::new();

    for service in services {
        let setting = match config.settings
            .iter()
            .find(|setting| setting.service_name == service.service_name) {
            Some(setting) => setting,
            None => return Err(Error::new(ErrorKind::NotFound, format!("Specified service: {} not found", service.service_name)))
        };

        // populate all new parameter values to be rendered
        for parameter in &service.parameters {
            data.insert(parameter.name.clone(), parameter.value.clone());
        }

        let mut source_template = match File::open(&setting.template) {
            Ok(file) => file,
            Err(err) => {
                return Err(Error::new(ErrorKind::InvalidData, format!("Could not open template file: {}. {}", &setting.template, err)))
            }
        };

        let mut output_file = match File::create(&setting.rendered) {
            Ok(file) => file,
            Err(err) => {
                return Err(Error::new(ErrorKind::Other, format!("Could not create render file: {}.  {}", &setting.rendered, err)));
            }
        };

        if let Err(err) = handlebars.render_template_source_to_write(
            &mut source_template,
            &data,
            &mut output_file) {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Could not render template file: {} -> render file: {}. {}", setting.template, setting.rendered, err)));
        }

//        let cmd_parts = setting.reload_cmd.split(" ")
//            .map(|entry| entry.to_owned())
//            .collect::<Vec<String>>();

//        if let Err(err) = Command::new(&cmd_parts[0]).args(&cmd_parts[1..]).spawn() {
//            return Err(Error::new(ErrorKind::Other, format!("Could not execute reload command: {}.  {}", setting.reload_cmd, err)));
//        }
    }

    Ok(())
}

pub fn parse_configuration(config_path: &Path) -> Config {
    let config_file = File::open(config_path)
        .expect(&format!("Could not open config file at path: {}", config_path.display()));
    let config: Config = serde_json::from_reader(config_file)
        .expect(&format!("Could not parse config file at path: {}", config_path.display()));

    for setting in &config.settings {
        if setting.keys.is_empty() {
            eprintln!("You must specify at least one key for service: {}", setting.service_name);
            std::process::exit(1);
        }
    }

    config
}


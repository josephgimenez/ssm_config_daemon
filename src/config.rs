use {
    crate::error::DaemonError,
    handlebars::Handlebars,
    serde_json::value::Map,
    std::{fs::File, path::Path, process::Command},
};

#[derive(Deserialize, Clone)]
pub struct Service {
    pub service_name: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Deserialize, Clone)]
pub struct Parameter {
    pub ps_path: String,
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Deserialize)]
pub struct Config {
    pub settings: Vec<Setting>,
}

#[derive(Deserialize)]
pub struct Setting {
    pub service_name: String,
    pub keys: Vec<String>,
    pub template: String,
    pub rendered: String,
    pub reload_cmd: String,
}

pub fn render_config(services: &Vec<Service>, config: &Config) -> Result<(), DaemonError> {
    let handlebars = Handlebars::new();
    let mut data = Map::new();

    for service in services {
        let setting = match config
            .settings
            .iter()
            .find(|setting| setting.service_name == service.service_name)
        {
            Some(setting) => setting,
            None => {
                return Err(DaemonError::SettingNotFound {
                    setting: service.service_name.clone(),
                })
            }
        };

        // populate all new parameter values to be rendered
        for parameter in &service.parameters {
            data.insert(parameter.name.clone(), parameter.value.clone());
        }

        let mut source_template = match File::open(&setting.template) {
            Ok(template) => template,
            Err(err) => {
                return Err(DaemonError::TemplateOpen {
                    template_path: setting.template.clone(),
                    error: err,
                })
            }
        };

        let mut output_file = match File::create(&setting.rendered) {
            Ok(output) => output,
            Err(err) => {
                return Err(DaemonError::CreateRender {
                    render_output_path: setting.rendered.clone(),
                    error: err,
                })
            }
        };

        if let Err(error) = handlebars.render_template_source_to_write(
            &mut source_template,
            &data,
            &mut output_file,
        ) {
            return Err(DaemonError::RenderTemplate {
                template_path: setting.template.to_owned(),
                error,
            });
        }

        if !setting.reload_cmd.is_empty() {
            let cmd_parts = setting
                .reload_cmd
                .split(" ")
                .map(|entry| entry.to_owned())
                .collect::<Vec<String>>();

            if let Err(error) = Command::new(&cmd_parts[0]).args(&cmd_parts[1..]).spawn() {
                return Err(DaemonError::ReloadCmd {
                    reload_cmd: setting.reload_cmd.clone(),
                    error,
                });
            }
        }
    }

    Ok(())
}

pub fn parse_configuration(config_path: &Path) -> Result<Config, DaemonError> {
    let config_file = match File::open(config_path) {
        Ok(config_file) => config_file,
        Err(err) => {
            return Err(DaemonError::Config {
                name: config_path.display().to_string(),
                error: err,
            })
        }
    };

    let config: Config = match serde_json::from_reader(config_file) {
        Ok(config) => config,
        Err(err) => {
            return Err(DaemonError::Json {
                config_name: config_path.display().to_string(),
                error: err,
            })
        }
    };

    for setting in &config.settings {
        if setting.keys.is_empty() {
            eprintln!(
                "You must specify at least one key for service: {}",
                setting.service_name
            );
            std::process::exit(1);
        }
    }

    Ok(config)
}

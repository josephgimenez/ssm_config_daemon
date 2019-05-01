use {
    crate::config::{Config, Parameter, Service},
    rusoto_core::Region,
    rusoto_ssm::Ssm,
    rusoto_ssm::{
        GetParameterRequest,
        SsmClient as AwsSSMClient,
    },
    std::io::{Error, ErrorKind},
};

pub struct SsmClient {
    client: AwsSSMClient
}

impl SsmClient {
    pub fn new() -> SsmClient {
        SsmClient {
            client: AwsSSMClient::new(Region::UsEast1)
        }
    }

    pub fn get_parameter(&self, key_path: &str) -> Result<String, std::io::Error> {
        match self.client.get_parameter(GetParameterRequest {
            name: key_path.to_owned(),
            with_decryption: Some(false),
        }).sync() {
            Ok(result) => {
                match result.parameter {
                    Some(parameter) => {
                       match parameter.value {
                           Some(value) => return Ok(value),
                           None => Err(Error::new(ErrorKind::NotFound, format!("No value found for key: {}", &key_path)))
                       }
                    }
                    None => {
                        Err(Error::new(ErrorKind::NotFound, format!("Could not locate key: {}", &key_path)))
                    }
                }
            },
            Err(err) => {
                Err(Error::new(ErrorKind::PermissionDenied, format!("Could not read parameter value: {}. {}", key_path, err)))
            }
        }
    }

    pub fn get_parameters(&self, config: &Config) -> Vec<Service> {
        let mut config_parameters = Vec::<Service>::new();

        for setting in &config.settings {
            let mut setting_parameters = Vec::<Parameter>::new();

            for key in &setting.keys {
                match self.get_parameter(&key) {
                    Ok(value) => {
                        let parameter_index = key.rfind("/").unwrap();
                        let parameter_name = &key[parameter_index+1..key.len()];

                        setting_parameters.push(
                            Parameter {
                                ps_path: key.clone(),
                                name: parameter_name.to_owned(),
                                value: serde_json::to_value(value).unwrap()
                            }
                        );
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    }
                };
            }

            config_parameters.push(
                Service {
                    service_name: setting.service_name.clone(),
                    parameters: setting_parameters
                }
            )
        }

        config_parameters
    }
}


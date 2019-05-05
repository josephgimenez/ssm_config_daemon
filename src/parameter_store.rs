use {
    crate::config::{Config, Parameter, Service},
    crate::error::DaemonError,
    rusoto_core::{Region, RusotoError, RusotoError::Service as RusotoService},
    rusoto_ssm::{
        GetParameterError,
        GetParameterError::{ParameterNotFound, ParameterVersionNotFound},
        GetParameterRequest, Ssm, SsmClient as AwsSSMClient,
    },
};

pub struct SsmClient {
    client: AwsSSMClient,
}

impl SsmClient {
    pub fn new() -> SsmClient {
        SsmClient {
            client: AwsSSMClient::new(Region::UsEast1),
        }
    }

    pub fn get_parameter(&self, key_path: &str) -> Result<String, RusotoError<GetParameterError>> {
        match self
            .client
            .get_parameter(GetParameterRequest {
                name: key_path.to_owned(),
                with_decryption: Some(false),
            })
            .sync()
        {
            Ok(parameter_result) => Ok(parameter_result.parameter.unwrap().value.unwrap()),
            Err(err) => Err(err),
        }
    }

    pub fn get_parameters(&self, config: &Config) -> Result<Vec<Service>, DaemonError> {
        let mut config_parameters = Vec::<Service>::new();

        for setting in &config.settings {
            let mut setting_parameters = Vec::<Parameter>::new();

            for key in &setting.keys {
                match self.get_parameter(&key) {
                    Ok(value) => {
                        let parameter_index = key.rfind("/").unwrap();
                        let parameter_name = &key[parameter_index + 1..key.len()];

                        setting_parameters.push(Parameter {
                            ps_path: key.clone(),
                            name: parameter_name.to_owned(),
                            value: serde_json::to_value(value).unwrap(),
                        });
                    }
                    Err(err) => {
                        eprintln!("{}", err);

                        match err {
                            RusotoService(ParameterNotFound(_))
                            | RusotoService(ParameterVersionNotFound(_)) => continue,
                            _ => {
                                return Err(DaemonError::SsmGet {
                                    key_path: key.clone(),
                                    error: err,
                                })
                            }
                        }
                    }
                }
            }

            config_parameters.push(Service {
                service_name: setting.service_name.clone(),
                parameters: setting_parameters,
            })
        }

        Ok(config_parameters)
    }
}

use {
    handlebars::TemplateRenderError,
    rusoto_core::RusotoError,
    rusoto_ssm::GetParameterError,
    serde_json::error::Error as JsonError,
    std::io::Error as IOError
};

#[derive(Debug, Fail, From)]
pub enum DaemonError {
    #[fail(display = "Could not open configuration file: {}. {}", name, error)]
    Config {
        name: String,
        error: IOError
    },
    #[fail(display = "Error parsing json in configuration file: {}. {}", config_name, error)]
    Json {
        config_name: String,
        error: JsonError
    },
    #[fail(display = "Error getting key at path: {}.  {}", key_path, error)]
    SsmGet {
        key_path: String,
        error: RusotoError<GetParameterError>
    },
    #[fail(display = "Could not find specified setting: {}.", setting)]
    SettingNotFound {
        setting: String
    },
    #[fail(display = "Error opening template: {}.  {}", template_path, error)]
    TemplateOpen {
        template_path: String,
        error: IOError
    },
    #[fail(display = "Error creating rendered template: {}.  {}", render_output_path, error)]
    CreateRender {
        render_output_path: String,
        error: IOError
    },
    #[fail(display = "Error rendering template: {}.  {}", template_path, error)]
    RenderTemplate {
        template_path: String,
        error: TemplateRenderError,
    },
    #[fail(display = "Error running reload command: {}.  {}", reload_cmd, error)]
    ReloadCmd {
        reload_cmd: String,
        error: IOError
    },
    #[fail(display = "Error describing kinesis stream: {}", error)]
    KinesisDescribeStream {
        error: RusotoError<rusoto_kinesis::DescribeStreamError>
    },
    #[fail(display = "Error getting kinesis shard iterator: {}", error)]
    KinesisShardIterator {
        error: RusotoError<rusoto_kinesis::GetShardIteratorError>
    },
    #[fail(display = "Error getting kinesis records: {}", error)]
    KinesisGetRecords {
        error: RusotoError<rusoto_kinesis::GetRecordsError>
    },
    #[fail(display = "Error decoding kinesis record value: {}", error)]
    KinesisRecordUTF8Decode {
        error: std::str::Utf8Error
    },
    #[fail(display = "Error decoding JSON from kinesis record value: {}", error)]
    KinesisRecordJsonDecode {
        error: JsonError
    }

}


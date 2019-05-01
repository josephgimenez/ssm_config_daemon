#[macro_use] extern crate serde_derive;

mod config;
mod kinesis_consumer;
mod parameter_store;

use {
    clap::{Arg, App},
    config::*,
    daemonize::Daemonize,
    kinesis_consumer::KinesisConsumerClient,
    parameter_store::SsmClient,
    serde_json::Value,
    std::{
        fs::File,
        path::Path,
        str,
        thread::sleep,
        time::Duration,
    },
};

fn main() {
    let matches = App::new("SSM Config Daemon")
        .version("1.1")
        .author("joseph.gimenez@snagajob.com")
        .arg(Arg::with_name("config")
                 .short("c")
                 .help("Specify path to SSM Config configuration file")
                 .takes_value(true)
                 .required(true))
        .get_matches();

    let config_path = Path::new(matches.value_of("config").unwrap());

    let stdout = File::create("/tmp/ssm_config.log").unwrap();
    let stderr = File::create("/tmp/ssm_config.err").unwrap();

    let daemonize = Daemonize::new()
            .pid_file("/tmp/ssm_config.pid")
            .working_directory("/tmp")
            .stdout(stdout)
            .stderr(stderr);

    if let Err(err) = daemonize.start() {
        eprintln!("Error starting as daemon process: {}", err);
        std::process::exit(1);
    }

    println!("Daemon successfully started.");

    let config =  parse_configuration(&config_path);

    let ssm_client = SsmClient::new();
    let service_parameters = ssm_client.get_parameters(&config);

    match render_config( &service_parameters, &config) {
        Ok(()) => {
            let mut kinesis_client = KinesisConsumerClient::new(config, service_parameters);
            match kinesis_client.start_event_loop() {
                Ok(()) => {},
                Err(err) => eprintln!("{}", err)
            }
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

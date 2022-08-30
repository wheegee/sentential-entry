use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ssm::{Client, Region};
use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::process::{Command, ExitStatus};

/// Wrapper for sententially-driven execution
#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Options {
    /// Verbose mode
    #[clap(short, long)]
    verbose: bool,

    /// AWS region to use
    #[clap(short, long, env = "AWS_REGION")]
    region: Option<String>,

    /// SSM prefix to filter
    prefix: String,

    /// Command to execute
    command: Vec<String>,
}

/// Fetch parameters in given SSM prefix.
///
/// Return parameters as `HashMap`.
async fn fetch_parameters(
    client: &Client,
    prefix: &str,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut parameters = HashMap::new();

    let resp = client
        .get_parameters_by_path()
        .path(prefix)
        .with_decryption(true)
        .send()
        .await?;

    for parameter in resp.parameters.unwrap().iter() {
        parameters.insert(
            parameter.name().unwrap().to_string().replace(prefix, ""),
            parameter.value().unwrap().to_string(),
        );
    }

    Ok(parameters)
}

/// Execute command with given parameters injected into the environment.
///
/// Return exit status as `ExitStatus`.
async fn execute_with_env(
    command: &[String],
    env: &HashMap<String, String>,
) -> Result<ExitStatus, std::io::Error> {
    Command::new(&command[0])
        .args(&command[1..])
        .envs(env)
        .spawn()?
        .wait()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Options {
        verbose,
        region,
        prefix,
        command,
    } = Options::parse();

    // AWS configuration
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    // Verbose output
    if verbose {
        println!("Region: {}", shared_config.region().unwrap());
    }

    let parameters = match fetch_parameters(&client, &prefix).await {
        Ok(parameters) => parameters,
        Err(error) => panic!(
            "error: failed to retrieve parameters under `{}`: {:?}",
            &prefix, error
        ),
    };

    execute_with_env(&command, &parameters).await?;

    Ok(())
}

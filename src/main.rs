use anyhow::{Context, Result};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_sts::config::Region;
use aws_sdk_sts::operation::assume_role::builders::AssumeRoleFluentBuilder;
use aws_sdk_sts::types::PolicyDescriptorType;
use aws_sdk_sts::Client;
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
use hyper::client::HttpConnector;
// use aws_smithy_runtime_api::client::behavior_version::BehaviorVersion;
// use aws_smithy_runtime_api::client::http::HttpConnector;
use serde::Serialize;

use hyper::Uri;
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use tracing_subscriber::EnvFilter;

use crate::credential_file::CredentialFile;
use crate::settings::Cmdline;

mod credential_file;
mod settings;

#[derive(Serialize)]
#[allow(non_snake_case)]
struct CredentialProcessOutput {
    Version: i32,
    AccessKeyId: String,
    SecretAccessKey: String,
    SessionToken: String,
    Expiration: String,
}

impl CredentialProcessOutput {
    fn from_credentials(credentials: &aws_sdk_sts::types::Credentials) -> Self {
        CredentialProcessOutput {
            Version: 1,
            AccessKeyId: credentials.access_key_id().to_string(),
            SecretAccessKey: credentials.secret_access_key().to_string(),
            SessionToken: credentials.session_token().to_string(),
            Expiration: credentials.expiration().to_string(),
        }
    }
}

macro_rules! vprintln {
    ($cmdline:expr, $($arg:tt)*) => {
        if $cmdline.verbose {
            println!($($arg)*);
        }
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmdline = Cmdline::parse();

    if cmdline.verbose {
        // Enable verbose tracing for AWS SDK HTTP traffic
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new("trace"))
            .init();
    }

    vprintln!(&cmdline, "Assuming role {}", cmdline.role_arn);

    let sts_client = build_sts_client(&cmdline).await?;
    let assume_role_request = build_assume_role_request(sts_client, &cmdline);
    let result = assume_role_request
        .send()
        .await
        .context("assume role failed")?;

    let credential_filename = cmdline.determine_credential_file()?;
    vprintln!(
        &cmdline,
        "Assume role succeeded, saving credentials to {}",
        credential_filename.display()
    );

    let credentials = result
        .credentials()
        .with_context(|| "no credentials in response")?;

    if cmdline.credential_process {
        let output = CredentialProcessOutput::from_credentials(credentials);
        println!(
            "{}",
            serde_json::to_string(&output).context("failed to serialize credentials")?
        );
    } else {
        let mut credential_file = CredentialFile::load(&credential_filename)?;
        credential_file.set_credentials(&cmdline.dest_profile, credentials);
        credential_file.save(&credential_filename)?;
    }

    Ok(())
}

async fn build_sts_client(cmdline: &Cmdline) -> Result<Client> {
    let region_provider = RegionProviderChain::first_try(cmdline.region.clone().map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-east-1"));
    let mut config_loader = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);

    if let Some(profile_name) = &cmdline.profile {
        config_loader = config_loader.profile_name(profile_name);
    }

    let config = config_loader.load().await;

    let mut sts_config_builder = aws_sdk_sts::config::Builder::from(&config);
    if let Some(proxy_uri) = &cmdline.proxy {
        let proxy = Proxy::new(
            Intercept::All,
            Uri::try_from(proxy_uri).context("invalid proxy_uri")?,
        );
        let connector = HttpConnector::new();
        let proxy_connector = ProxyConnector::from_proxy(connector, proxy)?;
        let http_client = HyperClientBuilder::new().build(proxy_connector);
        sts_config_builder = sts_config_builder.http_client(http_client);
    }
    let sts_config = sts_config_builder.build();

    let sts_client = Client::from_conf(sts_config);
    Ok(sts_client)
}

fn build_assume_role_request(client: Client, cmdline: &Cmdline) -> AssumeRoleFluentBuilder {
    let mut builder = client.assume_role();

    if let Some(duration_seconds) = cmdline.duration {
        builder = builder.duration_seconds(duration_seconds);
    }

    if let Some(external_id) = &cmdline.external_id {
        builder = builder.external_id(external_id);
    }

    if let Some(mfa_serial_number) = &cmdline.mfa_serial_number {
        builder = builder.serial_number(mfa_serial_number);
    }

    if let Some(mfa_token) = &cmdline.mfa_token {
        builder = builder.token_code(mfa_token);
    }

    if let Some(policy_arns) = &cmdline.policies {
        for policy_arn in policy_arns {
            builder = builder.policy_arns(
                PolicyDescriptorType::builder()
                    .arn(policy_arn.clone())
                    .build(),
            );
        }
    }

    if let Some(policy_json) = &cmdline.policy_json {
        builder = builder.policy(policy_json);
    }

    builder = builder.role_arn(&cmdline.role_arn);
    builder = builder.role_session_name(&cmdline.session_name);
    builder
}

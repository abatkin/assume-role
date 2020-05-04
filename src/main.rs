use anyhow::{Context, Result};
use rusoto_core::credential::{ChainProvider, ProfileProvider};
use rusoto_core::{Client, HttpClient};
use rusoto_sts::{AssumeRoleRequest, PolicyDescriptorType, Sts, StsClient};

use crate::credential_file::CredentialFile;
use crate::settings::Cmdline;

mod credential_file;
mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    let cmdline = Cmdline::parse();

    let sts_client = build_sts_client(&cmdline)?;
    let assume_role_request = build_assume_role_request(&cmdline);
    let result = sts_client.assume_role(assume_role_request).await?;

    let credentials = result
        .credentials
        .with_context(|| "no credentials in response")?;

    let credential_filename = cmdline.determine_credential_file()?;

    let mut credential_file = CredentialFile::load(&credential_filename)?;
    credential_file.set_credentials(&cmdline.dest_profile, &credentials);
    credential_file.save(&credential_filename)?;

    Ok(())
}

fn build_sts_client(cmdline: &Cmdline) -> Result<StsClient> {
    let credential_provider =
        build_credential_provider(&cmdline.profile, &cmdline.credential_file)?;

    let http_client = HttpClient::new().context("failed to create request dispatcher")?;
    let client = Client::new_with(credential_provider, http_client);
    let sts_client = StsClient::new_with_client(client, cmdline.region.clone());

    Ok(sts_client)
}

fn build_credential_provider(
    profile: &Option<String>,
    credential_file: &Option<String>,
) -> Result<ChainProvider> {
    let mut profile_provider = ProfileProvider::new()?;
    if let Some(profile_name) = profile {
        profile_provider.set_profile(profile_name);
    }
    if let Some(filename) = credential_file {
        profile_provider.set_file_path(filename);
    }
    Ok(ChainProvider::with_profile_provider(profile_provider))
}

fn build_assume_role_request(cmdline: &Cmdline) -> AssumeRoleRequest {
    let policies = match &cmdline.policies {
        Some(policies) => Some(
            policies
                .iter()
                .map(|arn| PolicyDescriptorType {
                    arn: Some(arn.clone()),
                })
                .collect(),
        ),
        None => None,
    };

    AssumeRoleRequest {
        duration_seconds: cmdline.duration.map(|v| v as i64),
        external_id: cmdline.external_id.clone(),
        policy: cmdline.policy_json.clone(),
        policy_arns: policies,
        role_arn: cmdline.role_arn.clone(),
        role_session_name: cmdline.session_name.clone(),
        serial_number: cmdline.mfa_serial_number.clone(),
        token_code: cmdline.mfa_token.clone(),
        ..AssumeRoleRequest::default()
    }
}

use std::path::Path;

use anyhow::{Context, Result};
use ini::Ini;
use rusoto_sts::Credentials;

pub struct CredentialFile {
    ini: Ini,
}

impl CredentialFile {
    pub fn load<P: AsRef<Path>>(filename: P) -> Result<CredentialFile> {
        let path = filename.as_ref();
        let ini = Ini::load_from_file(path)
            .with_context(|| format!("unable to load credential file {}", path.display()))?;
        Ok(CredentialFile { ini })
    }

    pub fn set_credentials(&mut self, profile: &str, credentials: &Credentials) {
        self.ini.delete(Some(profile));
        self.ini
            .with_section(Some(profile))
            .set("aws_access_key_id", &credentials.access_key_id)
            .set("aws_secret_access_key", &credentials.secret_access_key)
            .set("aws_session_token", &credentials.session_token);
    }

    pub fn save<P: AsRef<Path>>(&self, filename: P) -> Result<()> {
        let path = filename.as_ref();
        self.ini
            .write_to_file(path)
            .with_context(|| format!("unable to save credential file {}", path.display()))
    }
}

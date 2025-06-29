use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use aws_sdk_sts::types::Credentials;
use ini::Ini;

pub struct CredentialFile {
    ini: Ini,
}

impl CredentialFile {
    pub fn load<P: AsRef<Path>>(filename: P) -> Result<CredentialFile> {
        let path = filename.as_ref();
        let ini = if fs::metadata(path).is_err() {
            Ini::new()
        } else {
            match Ini::load_from_file(path) {
                Ok(ini) => ini,
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("unable to load credential file {}", path.display())
                    })
                }
            }
        };
        Ok(CredentialFile { ini })
    }

    pub fn set_credentials(&mut self, profile: &str, credentials: &Credentials) {
        self.ini.delete(Some(profile));
        self.ini
            .with_section(Some(profile))
            .set("aws_access_key_id", credentials.access_key_id())
            .set("aws_secret_access_key", credentials.secret_access_key())
            .set("aws_session_token", credentials.session_token());
    }

    pub fn save<P: AsRef<Path>>(&self, filename: P) -> Result<()> {
        let path = filename.as_ref();
        self.ini
            .write_to_file(path)
            .with_context(|| format!("unable to save credential file {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_missing_file_returns_empty() {
        let tmp = std::env::temp_dir().join("assume_role_test_missing.ini");
        let _ = fs::remove_file(&tmp);
        let cred = CredentialFile::load(&tmp).expect("load should succeed");
        assert!(cred.ini.section(None::<String>).is_none());
    }
}

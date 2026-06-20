use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransferOptions {
    #[serde(default)]
    pub smart_sync: bool,
    #[serde(default)]
    pub strict_hash_sync: bool,
    #[serde(default = "default_true")]
    pub verify_after_copy: bool,
    #[serde(default)]
    pub stop_on_integrity_error: bool,
    #[serde(default)]
    pub encrypt_output: bool,
    #[serde(default)]
    pub encrypt_password: Option<String>,
    #[serde(default)]
    pub remember_encrypt_password: bool,
    #[serde(default)]
    pub remove_files_after_pack: bool,
    #[serde(default)]
    pub pack_filename: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            smart_sync: false,
            strict_hash_sync: false,
            verify_after_copy: true,
            stop_on_integrity_error: false,
            encrypt_output: false,
            encrypt_password: None,
            remember_encrypt_password: false,
            remove_files_after_pack: false,
            pack_filename: None,
        }
    }
}

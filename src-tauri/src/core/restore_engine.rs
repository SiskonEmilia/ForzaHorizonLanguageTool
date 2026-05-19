#[derive(Debug, Clone, serde::Serialize)]
pub struct RestoreResult {
    pub success: bool,
    pub message: String,
}

// TODO: implement
pub fn execute_restore(_backup_path: &std::path::Path) -> RestoreResult {
    RestoreResult {
        success: false,
        message: "not implemented".into(),
    }
}

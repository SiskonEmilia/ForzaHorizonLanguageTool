use super::language_mapper::ApplyPlan;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
    pub backup_path: Option<String>,
    pub rolled_back: bool,
}

// TODO: implement
pub fn execute_apply(_plan: &ApplyPlan) -> ApplyResult {
    ApplyResult {
        success: false,
        message: "not implemented".into(),
        backup_path: None,
        rolled_back: false,
    }
}

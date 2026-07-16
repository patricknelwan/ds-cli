pub const MODELS: [&str; 2] = ["deepseek-v4-flash", "deepseek-v4-pro"];
pub const DEFAULT_MODEL: &str = MODELS[0];

pub fn is_supported(model: &str) -> bool {
    MODELS.contains(&model)
}

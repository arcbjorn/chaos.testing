pub mod go;
pub mod python;
pub mod rust_gen;

use crate::models::CapturedRequest;
use anyhow::Result;

pub trait TestGenerator {
    fn generate(&self, requests: &[CapturedRequest]) -> Result<String>;
    fn file_extension(&self) -> &str;
}

pub fn get_generator(language: &str, framework: Option<&str>) -> Result<Box<dyn TestGenerator>> {
    match language.to_lowercase().as_str() {
        "python" | "py" => {
            let framework = framework.unwrap_or("pytest");
            Ok(Box::new(python::PythonGenerator::new(framework)))
        }
        "go" | "golang" => Ok(Box::new(go::GoGenerator::new())),
        "rust" | "rs" => Ok(Box::new(rust_gen::RustGenerator::new())),
        _ => anyhow::bail!("Unsupported language: {}", language),
    }
}

pub mod go;
pub mod python;
pub mod rust_gen;

#[cfg(test)]
mod tests;

use crate::models::CapturedRequest;
use anyhow::Result;
use go::GoGenerator;
use python::PythonGenerator;
use rust_gen::RustGenerator;

pub trait TestGenerator {
    fn generate(&self, requests: &[CapturedRequest]) -> Result<String>;
    fn file_extension(&self) -> &str;
}

pub fn get_generator(language: &str, framework: Option<&str>) -> Result<Box<dyn TestGenerator>> {
    match language.to_lowercase().as_str() {
        "python" | "py" | "auto" => {
            let framework = framework.unwrap_or("pytest");
            Ok(Box::new(PythonGenerator::new(framework)))
        }
        "go" | "golang" => Ok(Box::new(GoGenerator::new())),
        "rust" | "rs" => Ok(Box::new(RustGenerator::new())),
        _ => anyhow::bail!("Unsupported language: {}", language),
    }
}

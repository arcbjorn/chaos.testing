use crate::generators::TestGenerator;
use crate::models::CapturedRequest;
use anyhow::Result;
use std::collections::HashMap;

pub struct PythonGenerator;

impl PythonGenerator {
    pub fn new(_framework: &str) -> Self {
        Self
    }

    fn group_by_endpoint<'a>(
        &self,
        requests: &'a [CapturedRequest],
    ) -> HashMap<String, Vec<&'a CapturedRequest>> {
        let mut grouped: HashMap<String, Vec<&'a CapturedRequest>> = HashMap::new();

        for req in requests {
            let key = format!("{} {}", req.request.method, req.request.uri);
            grouped.entry(key).or_default().push(req);
        }

        grouped
    }

    fn sanitize_test_name(&self, name: &str) -> String {
        name.to_lowercase()
            .replace(['/', '-'], "_")
            .replace('?', "")
            .replace(['&', '='], "_")
            .trim_matches('_')
            .to_string()
    }
}

impl TestGenerator for PythonGenerator {
    fn generate(&self, requests: &[CapturedRequest]) -> Result<String> {
        let mut output = String::new();

        output.push_str("import pytest\n");
        output.push_str("import requests\n");
        output.push_str("from typing import Dict, Any\n\n");
        output.push_str("BASE_URL = \"http://localhost:8080\"\n\n");

        let grouped = self.group_by_endpoint(requests);

        for (endpoint, reqs) in grouped.iter() {
            let first_req = reqs[0];
            let test_name = self.sanitize_test_name(endpoint);

            output.push_str(&format!("def test_{}():\n", test_name));
            output.push_str(&format!("    \"\"\"Test {} endpoint\"\"\"\n", endpoint));

            let method_lower = first_req.request.method.to_lowercase();
            output.push_str(&format!(
                "    response = requests.{}(f\"{{BASE_URL}}{}\"",
                method_lower, first_req.request.uri
            ));

            if !first_req.request.headers.is_empty() {
                output.push_str(",\n        headers={\n");
                for (key, value) in &first_req.request.headers {
                    if key != "host" && key != "content-length" {
                        output.push_str(&format!("            \"{}\": \"{}\",\n", key, value));
                    }
                }
                output.push_str("        }");
            }

            output.push_str(")\n\n");

            if let Some(response) = &first_req.response {
                output.push_str(&format!(
                    "    assert response.status_code == {}\n",
                    response.status_code
                ));
            } else {
                output.push_str("    assert response.status_code < 500\n");
            }

            output.push_str(&format!("    # Called {} times in capture\n", reqs.len()));
            output.push_str("\n\n");
        }

        if output.is_empty() {
            output.push_str("# No requests captured\n");
        }

        Ok(output)
    }

    fn file_extension(&self) -> &str {
        "py"
    }
}

use crate::generators::TestGenerator;
use crate::models::CapturedRequest;
use anyhow::Result;
use std::collections::HashMap;

pub struct RustGenerator;

impl RustGenerator {
    pub fn new() -> Self {
        Self
    }

    fn group_by_endpoint<'a>(&self, requests: &'a [CapturedRequest]) -> HashMap<String, Vec<&'a CapturedRequest>> {
        let mut grouped: HashMap<String, Vec<&'a CapturedRequest>> = HashMap::new();

        for req in requests {
            let key = format!("{} {}", req.request.method, req.request.uri);
            grouped.entry(key).or_default().push(req);
        }

        grouped
    }

    fn sanitize_test_name(&self, name: &str) -> String {
        name.to_lowercase()
            .replace('/', "_")
            .replace('-', "_")
            .replace('?', "")
            .replace('&', "_")
            .replace('=', "_")
            .trim_matches('_')
            .to_string()
    }
}

impl TestGenerator for RustGenerator {
    fn generate(&self, requests: &[CapturedRequest]) -> Result<String> {
        let mut output = String::new();

        output.push_str("#[cfg(test)]\n");
        output.push_str("mod tests {\n");
        output.push_str("    use reqwest;\n\n");
        output.push_str("    const BASE_URL: &str = \"http://localhost:8080\";\n\n");

        let grouped = self.group_by_endpoint(requests);

        for (endpoint, reqs) in grouped.iter() {
            let first_req = reqs[0];
            let test_name = self.sanitize_test_name(endpoint);

            output.push_str("    #[tokio::test]\n");
            output.push_str(&format!("    async fn test_{}() {{\n", test_name));
            output.push_str(&format!("        // Test {} endpoint\n", endpoint));

            let method_lower = first_req.request.method.to_lowercase();
            output.push_str("        let client = reqwest::Client::new();\n");
            output.push_str(&format!(
                "        let response = client.{}(format!(\"{{}}{})\", BASE_URL))\n",
                method_lower, first_req.request.uri
            ));

            for (key, value) in &first_req.request.headers {
                if key != "host" && key != "content-length" {
                    output.push_str(&format!(
                        "            .header(\"{}\", \"{}\")\n",
                        key, value
                    ));
                }
            }

            output.push_str("            .send()\n");
            output.push_str("            .await\n");
            output.push_str("            .expect(\"Failed to send request\");\n\n");

            if let Some(response) = &first_req.response {
                output.push_str(&format!(
                    "        assert_eq!(response.status().as_u16(), {});\n",
                    response.status_code
                ));
            } else {
                output.push_str("        assert!(response.status().as_u16() < 500);\n");
            }

            output.push_str(&format!("        // Called {} times in capture\n", reqs.len()));
            output.push_str("    }\n\n");
        }

        output.push_str("}\n");

        Ok(output)
    }

    fn file_extension(&self) -> &str {
        "rs"
    }
}

use crate::generators::TestGenerator;
use crate::models::CapturedRequest;
use anyhow::Result;
use std::collections::HashMap;

pub struct GoGenerator;

impl GoGenerator {
    pub fn new() -> Self {
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
        name.split_whitespace()
            .map(|s| {
                let s = s.replace(['/', '-'], "_");
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

impl TestGenerator for GoGenerator {
    fn generate(&self, requests: &[CapturedRequest]) -> Result<String> {
        let mut output = String::new();

        output.push_str("package main\n\n");
        output.push_str("import (\n");
        output.push_str("\t\"net/http\"\n");
        output.push_str("\t\"testing\"\n");
        output.push_str(")\n\n");
        output.push_str("const baseURL = \"http://localhost:8080\"\n\n");

        let grouped = self.group_by_endpoint(requests);

        for (endpoint, reqs) in grouped.iter() {
            let first_req = reqs[0];
            let test_name = self.sanitize_test_name(endpoint);

            output.push_str(&format!("func Test{}(t *testing.T) {{\n", test_name));
            output.push_str(&format!("\t// Test {} endpoint\n", endpoint));
            output.push_str(&format!(
                "\treq, err := http.NewRequest(\"{}\", baseURL+\"{}\", nil)\n",
                first_req.request.method, first_req.request.uri
            ));
            output.push_str("\tif err != nil {\n");
            output.push_str("\t\tt.Fatal(err)\n");
            output.push_str("\t}\n\n");

            for (key, value) in &first_req.request.headers {
                if key != "host" && key != "content-length" {
                    output.push_str(&format!("\treq.Header.Set(\"{}\", \"{}\")\n", key, value));
                }
            }

            output.push_str("\n\tclient := &http.Client{}\n");
            output.push_str("\tresp, err := client.Do(req)\n");
            output.push_str("\tif err != nil {\n");
            output.push_str("\t\tt.Fatal(err)\n");
            output.push_str("\t}\n");
            output.push_str("\tdefer resp.Body.Close()\n\n");

            if let Some(response) = &first_req.response {
                output.push_str(&format!(
                    "\tif resp.StatusCode != {} {{\n",
                    response.status_code
                ));
                output.push_str(&format!(
                    "\t\tt.Errorf(\"expected status {}, got %d\", resp.StatusCode)\n",
                    response.status_code
                ));
                output.push_str("\t}\n");
            } else {
                output.push_str("\tif resp.StatusCode >= 500 {\n");
                output.push_str("\t\tt.Errorf(\"server error: %d\", resp.StatusCode)\n");
                output.push_str("\t}\n");
            }

            output.push_str(&format!("\t// Called {} times in capture\n", reqs.len()));
            output.push_str("}\n\n");
        }

        if output
            == "package main\n\nimport (\n\t\"net/http\"\n\t\"testing\"\n)\n\nconst baseURL = \"http://localhost:8080\"\n\n"
        {
            output.push_str("// No requests captured\n");
        }

        Ok(output)
    }

    fn file_extension(&self) -> &str {
        "go"
    }
}

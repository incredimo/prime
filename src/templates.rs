use std::collections::HashMap;
use handlebars::Handlebars;
use serde_json::Value;
use anyhow::{Result, anyhow};

pub struct TaskTemplates {
    templates: HashMap<String, String>,
    handlebars: Handlebars<'static>,
}

impl TaskTemplates {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        let mut handlebars = Handlebars::new();
        
        // Python package installation
        templates.insert("install_python_package".to_string(), r#"
First, I'll check your Python environment:
```{.script execute="python --version"}
```

Then install the package:
```{.script execute="pip install {{package}}"}
```
"#.to_string());

        // Create project structure
        templates.insert("create_project".to_string(), r#"
Creating project structure for {{project_name}}:
```{.script save="{{project_name}}/README.md"}
# {{project_name}}

## Description
{{description}}
```

```{.script save="{{project_name}}/.gitignore"}
__pycache__/
*.pyc
.env
venv/
```

```{.script save="{{project_name}}/requirements.txt"}
# Dependencies for {{project_name}}
```
"#.to_string());

        // Node.js project setup
        templates.insert("setup_nodejs".to_string(), r#"
Initialize Node.js project:
```{.script execute="npm init -y"}
```

Install core dependencies:
```{.script execute="npm install {{#each dependencies}}{{this}} {{/each}}"}
```
"#.to_string());

        // Register templates with Handlebars
        for (name, template) in &templates {
            handlebars.register_template_string(name, template)
                .expect("Failed to register template");
        }
        
        Self { templates, handlebars }
    }
    
    pub fn render_template(&self, template_name: &str, data: &Value) -> Result<String> {
        if let Some(_) = self.templates.get(template_name) {
            Ok(self.handlebars.render(template_name, data)?)
        } else {
            Err(anyhow!("Template '{}' not found", template_name))
        }
    }
    
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
    
    pub fn add_template(&mut self, name: String, template: String) -> Result<()> {
        // Validate template first
        self.handlebars.register_template_string(&name, &template)?;
        self.templates.insert(name, template);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_template_rendering() {
        let templates = TaskTemplates::new();
        
        // Test Python package installation
        let data = json!({
            "package": "requests"
        });
        let result = templates.render_template("install_python_package", &data).unwrap();
        assert!(result.contains("pip install requests"));
        
        // Test project creation
        let data = json!({
            "project_name": "test-project",
            "description": "A test project"
        });
        let result = templates.render_template("create_project", &data).unwrap();
        assert!(result.contains("# test-project"));
        assert!(result.contains("A test project"));
    }
}
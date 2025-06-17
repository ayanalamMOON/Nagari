use crate::config::NagConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub struct DocGenerator {
    config: NagConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocModule {
    pub name: String,
    pub path: String,
    pub description: String,
    pub functions: Vec<DocFunction>,
    pub classes: Vec<DocClass>,
    pub constants: Vec<DocConstant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocFunction {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub parameters: Vec<DocParameter>,
    pub return_type: Option<String>,
    pub return_description: Option<String>,
    pub examples: Vec<String>,
    pub line_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocClass {
    pub name: String,
    pub description: String,
    pub methods: Vec<DocFunction>,
    pub properties: Vec<DocProperty>,
    pub inheritance: Vec<String>,
    pub line_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocParameter {
    pub name: String,
    pub param_type: Option<String>,
    pub description: String,
    pub default_value: Option<String>,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocProperty {
    pub name: String,
    pub prop_type: Option<String>,
    pub description: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocConstant {
    pub name: String,
    pub value: String,
    pub const_type: Option<String>,
    pub description: String,
    pub line_number: u32,
}

impl DocGenerator {
    pub fn new(config: &NagConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn generate(
        &self,
        source_dir: &Path,
        output_dir: &Path,
        format: &str,
        include_private: bool,
    ) -> Result<()> {
        std::fs::create_dir_all(output_dir)?;

        let modules = self.scan_modules(source_dir, include_private)?;

        match format {
            "html" => self.generate_html(&modules, output_dir)?,
            "markdown" => self.generate_markdown(&modules, output_dir)?,
            "json" => self.generate_json(&modules, output_dir)?,
            _ => anyhow::bail!("Unsupported format: {}", format),
        }

        Ok(())
    }

    fn scan_modules(&self, source_dir: &Path, include_private: bool) -> Result<Vec<DocModule>> {
        let mut modules = Vec::new();

        for entry in walkdir::WalkDir::new(source_dir) {
            let entry = entry?;
            if entry.file_type().is_file() &&
               entry.path().extension().and_then(|s| s.to_str()) == Some("nag") {

                let module = self.parse_module(entry.path(), include_private)?;
                modules.push(module);
            }
        }

        Ok(modules)
    }

    fn parse_module(&self, file_path: &Path, include_private: bool) -> Result<DocModule> {
        let content = std::fs::read_to_string(file_path)?;
        let relative_path = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut module = DocModule {
            name: relative_path.clone(),
            path: file_path.to_string_lossy().to_string(),
            description: String::new(),
            functions: Vec::new(),
            classes: Vec::new(),
            constants: Vec::new(),
        };

        // Parse module-level docstring
        module.description = self.extract_module_docstring(&content);

        // Parse functions
        module.functions = self.extract_functions(&content, include_private)?;

        // Parse classes
        module.classes = self.extract_classes(&content, include_private)?;

        // Parse constants
        module.constants = self.extract_constants(&content, include_private)?;

        Ok(module)
    }

    fn extract_module_docstring(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut docstring = String::new();
        let mut in_docstring = false;
        let mut quote_style = "";

        for line in lines {
            let trimmed = line.trim();

            if !in_docstring {
                if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                    in_docstring = true;
                    quote_style = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };

                    let content_after_quotes = trimmed.strip_prefix(quote_style).unwrap();
                    if content_after_quotes.ends_with(quote_style) {
                        // Single-line docstring
                        return content_after_quotes.strip_suffix(quote_style).unwrap().to_string();
                    } else {
                        docstring.push_str(content_after_quotes);
                        docstring.push('\n');
                    }
                }
            } else {
                if trimmed.ends_with(quote_style) {
                    let content_before_quotes = trimmed.strip_suffix(quote_style).unwrap();
                    docstring.push_str(content_before_quotes);
                    break;
                } else {
                    docstring.push_str(trimmed);
                    docstring.push('\n');
                }
            }
        }

        docstring.trim().to_string()
    }

    fn extract_functions(&self, content: &str, include_private: bool) -> Result<Vec<DocFunction>> {
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                if let Some(function) = self.parse_function(&lines, i, include_private)? {
                    functions.push(function);
                }
            }
        }

        Ok(functions)
    }

    fn parse_function(&self, lines: &[&str], start_line: usize, include_private: bool) -> Result<Option<DocFunction>> {
        let line = lines[start_line];
        let trimmed = line.trim();

        // Extract function signature
        let def_pos = if trimmed.starts_with("async def ") { 10 } else { 4 };
        let signature_part = &trimmed[def_pos..];

        if let Some(paren_pos) = signature_part.find('(') {
            let func_name = signature_part[..paren_pos].trim();

            // Skip private functions unless explicitly included
            if !include_private && func_name.starts_with('_') {
                return Ok(None);
            }

            let signature = trimmed.to_string();

            // Extract docstring
            let (description, parameters, return_info, examples) =
                self.extract_function_docstring(lines, start_line + 1)?;

            let function = DocFunction {
                name: func_name.to_string(),
                signature,
                description,
                parameters,
                return_type: return_info.0,
                return_description: return_info.1,
                examples,
                line_number: (start_line + 1) as u32,
            };

            return Ok(Some(function));
        }

        Ok(None)
    }

    fn extract_function_docstring(
        &self,
        lines: &[&str],
        start_line: usize,
    ) -> Result<(String, Vec<DocParameter>, (Option<String>, Option<String>), Vec<String>)> {
        let mut description = String::new();
        let mut parameters = Vec::new();
        let mut return_info = (None, None);
        let mut examples = Vec::new();

        // Look for docstring starting after function definition
        let mut current_line = start_line;
        while current_line < lines.len() {
            let line = lines[current_line].trim();

            if line.starts_with("\"\"\"") || line.starts_with("'''") {
                let quote_style = if line.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
                let mut docstring_lines = Vec::new();

                // Handle single-line docstring
                if line.len() > 3 && line.ends_with(quote_style) {
                    let content = line.strip_prefix(quote_style)
                        .unwrap()
                        .strip_suffix(quote_style)
                        .unwrap();
                    docstring_lines.push(content);
                } else {
                    // Multi-line docstring
                    current_line += 1;
                    while current_line < lines.len() {
                        let docline = lines[current_line].trim();
                        if docline.ends_with(quote_style) {
                            let content = docline.strip_suffix(quote_style).unwrap();
                            if !content.is_empty() {
                                docstring_lines.push(content);
                            }
                            break;
                        } else {
                            docstring_lines.push(docline);
                        }
                        current_line += 1;
                    }
                }

                // Parse docstring content
                self.parse_docstring_content(&docstring_lines, &mut description, &mut parameters, &mut return_info, &mut examples);
                break;
            } else if !line.is_empty() {
                break; // No docstring found
            }

            current_line += 1;
        }

        Ok((description, parameters, return_info, examples))
    }

    fn parse_docstring_content(
        &self,
        lines: &[&str],
        description: &mut String,
        parameters: &mut Vec<DocParameter>,
        return_info: &mut (Option<String>, Option<String>),
        examples: &mut Vec<String>,
    ) {
        let mut current_section = "description";
        let mut current_param: Option<DocParameter> = None;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.starts_with("Args:") || trimmed.starts_with("Parameters:") {
                current_section = "parameters";
                continue;
            } else if trimmed.starts_with("Returns:") {
                current_section = "returns";
                continue;
            } else if trimmed.starts_with("Examples:") {
                current_section = "examples";
                continue;
            }

            match current_section {
                "description" => {
                    if !description.is_empty() {
                        description.push('\n');
                    }
                    description.push_str(trimmed);
                }
                "parameters" => {
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Parameter format: "param_name (type): description"
                    if let Some(colon_pos) = trimmed.find(':') {
                        let param_part = &trimmed[..colon_pos].trim();
                        let desc_part = &trimmed[colon_pos + 1..].trim();

                        let mut param_name = param_part.to_string();
                        let mut param_type = None;

                        // Extract type from parentheses
                        if let Some(open_paren) = param_part.find('(') {
                            if let Some(close_paren) = param_part.find(')') {
                                param_name = param_part[..open_paren].trim().to_string();
                                param_type = Some(param_part[open_paren + 1..close_paren].trim().to_string());
                            }
                        }

                        let parameter = DocParameter {
                            name: param_name,
                            param_type,
                            description: desc_part.to_string(),
                            default_value: None,
                            optional: false,
                        };

                        parameters.push(parameter);
                    }
                }
                "returns" => {
                    if return_info.1.is_none() {
                        return_info.1 = Some(trimmed.to_string());
                    } else {
                        return_info.1 = Some(format!("{}\n{}", return_info.1.as_ref().unwrap(), trimmed));
                    }
                }
                "examples" => {
                    examples.push(trimmed.to_string());
                }
                _ => {}
            }
        }
    }

    fn extract_classes(&self, content: &str, include_private: bool) -> Result<Vec<DocClass>> {
        let mut classes = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("class ") {
                if let Some(class) = self.parse_class(&lines, i, include_private)? {
                    classes.push(class);
                }
            }
        }

        Ok(classes)
    }

    fn parse_class(&self, lines: &[&str], start_line: usize, include_private: bool) -> Result<Option<DocClass>> {
        let line = lines[start_line];
        let trimmed = line.trim();

        // Extract class name
        let class_part = &trimmed[6..]; // Skip "class "
        let class_name = if let Some(colon_pos) = class_part.find(':') {
            class_part[..colon_pos].trim()
        } else {
            class_part.trim()
        };

        // Skip private classes unless explicitly included
        if !include_private && class_name.starts_with('_') {
            return Ok(None);
        }

        // Extract docstring
        let (description, _, _, _) = self.extract_function_docstring(lines, start_line + 1)?;

        // Extract methods and properties
        let mut methods = Vec::new();
        let mut properties = Vec::new();
        let mut inheritance = Vec::new();

        // Extract inheritance from class definition
        if let Some(colon_pos) = class_part.find(':') {
            if let Some(paren_pos) = class_part.find('(') {
            if paren_pos < colon_pos {
                let inherit_part = &class_part[paren_pos + 1..colon_pos];
                if let Some(close_paren) = inherit_part.rfind(')') {
                let inherit_list = &inherit_part[..close_paren];
                inheritance = inherit_list
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                }
            }
            }
        }

        // Find class body and extract methods/properties
        let mut brace_level = 0;
        let mut in_class_body = false;
        let mut class_end_line = lines.len();

        for (i, line) in lines.iter().enumerate().skip(start_line) {
            let trimmed = line.trim();

            if i == start_line {
            in_class_body = true;
            continue;
            }

            // Track indentation to determine class body boundaries
            if in_class_body && !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
            class_end_line = i;
            break;
            }

            if in_class_body && i < class_end_line {
            // Extract methods (def statements within class)
            if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
                if let Some(method) = self.parse_function(lines, i, include_private)? {
                methods.push(method);
                }
            }

            // Extract properties (simple assignments or @property decorators)
            if trimmed.starts_with("@property") {
                // Look for the property definition in the next few lines
                for j in (i + 1)..std::cmp::min(i + 3, lines.len()) {
                let prop_line = lines[j].trim();
                if prop_line.starts_with("def ") {
                    let name = self.extract_property_name(prop_line);
                    if include_private || !name.starts_with('_') {
                        let property = DocProperty {
                            name,
                            prop_type: Some(self.extract_property_type(prop_line)),
                            description: self.extract_property_description(&lines.iter().map(|s| s.to_string()).collect::<Vec<_>>(), j).unwrap_or_default(),
                            readonly: true, // @property is readonly by default
                        };
                        properties.push(property);
                    }
                    break;
                }
                }
            } else if trimmed.contains(" = ") &&
                 trimmed.starts_with("self.") &&
                 !trimmed.contains("def ") {
                // Instance variable assignment
                if let Some(eq_pos) = trimmed.find(" = ") {
                let var_part = &trimmed[5..eq_pos].trim(); // Skip "self."
                if include_private || !var_part.starts_with('_') {
                    let property = DocProperty {
                        name: var_part.to_string(),
                        prop_type: Some(self.infer_type_from_assignment(&trimmed[eq_pos + 1..].trim())),
                        description: self.extract_property_description(&lines.iter().map(|s| s.to_string()).collect::<Vec<_>>(), i).unwrap_or_default(),
                        readonly: false,
                    };
                    properties.push(property);
                }
                }
            }
            }
        }
        let methods = Vec::new();
        let properties = Vec::new();
        let inheritance = Vec::new();

        let class = DocClass {
            name: class_name.to_string(),
            description,
            methods,
            properties,
            inheritance,
            line_number: (start_line + 1) as u32,
        };

        Ok(Some(class))
    }

    fn extract_constants(&self, content: &str, include_private: bool) -> Result<Vec<DocConstant>> {
        let mut constants = Vec::new();

        // Simple constant detection - uppercase variables at module level
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            if let Some(eq_pos) = trimmed.find('=') {
                let var_part = trimmed[..eq_pos].trim();
                let value_part = trimmed[eq_pos + 1..].trim();

                // Check if it's a constant (all uppercase)
                if var_part.chars().all(|c| c.is_uppercase() || c == '_') &&
                   var_part.chars().any(|c| c.is_alphabetic()) {

                    if !include_private && var_part.starts_with('_') {
                        continue;
                    }

                    let constant = DocConstant {
                        name: var_part.to_string(),
                        value: value_part.to_string(),
                        const_type: None, // TODO: Infer type
                        description: String::new(), // TODO: Extract from comments
                        line_number: (line_num + 1) as u32,
                    };

                    constants.push(constant);
                }
            }
        }

        Ok(constants)
    }

    fn generate_html(&self, modules: &[DocModule], output_dir: &Path) -> Result<()> {
        // Generate index.html
        let index_content = self.generate_html_index(modules)?;
        std::fs::write(output_dir.join("index.html"), index_content)?;

        // Generate individual module pages
        for module in modules {
            let module_content = self.generate_html_module(module)?;
            let filename = format!("{}.html", module.name);
            std::fs::write(output_dir.join(filename), module_content)?;
        }

        // Copy CSS file
        let css_content = include_str!("../../../assets/docs.css");
        std::fs::write(output_dir.join("style.css"), css_content)?;

        Ok(())
    }

    fn generate_html_index(&self, modules: &[DocModule]) -> Result<String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("    <title>Nagari Documentation</title>\n");
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("    <div class=\"container\">\n");
        html.push_str("        <h1>Nagari Documentation</h1>\n");
        html.push_str("        <div class=\"modules\">\n");

        for module in modules {
            html.push_str(&format!(
                "            <div class=\"module-card\">\n"));
            html.push_str(&format!(
                "                <h2><a href=\"{}.html\">{}</a></h2>\n",
                module.name, module.name
            ));
            html.push_str(&format!(
                "                <p>{}</p>\n",
                module.description
            ));
            html.push_str(&format!(
                "                <div class=\"stats\">\n"));
            html.push_str(&format!(
                "                    <span>{} functions</span>\n",
                module.functions.len()
            ));
            html.push_str(&format!(
                "                    <span>{} classes</span>\n",
                module.classes.len()
            ));
            html.push_str(&format!(
                "                </div>\n"));
            html.push_str("            </div>\n");
        }

        html.push_str("        </div>\n");
        html.push_str("    </div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }

    fn generate_html_module(&self, module: &DocModule) -> Result<String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>{} - Nagari Documentation</title>\n", module.name));
        html.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("    <div class=\"container\">\n");
        html.push_str("        <nav><a href=\"index.html\">← Back to Index</a></nav>\n");
        html.push_str(&format!("        <h1>Module: {}</h1>\n", module.name));
        html.push_str(&format!("        <p class=\"description\">{}</p>\n", module.description));

        // Functions
        if !module.functions.is_empty() {
            html.push_str("        <h2>Functions</h2>\n");
            for function in &module.functions {
                html.push_str(&self.generate_html_function(function)?);
            }
        }

        // Classes
        if !module.classes.is_empty() {
            html.push_str("        <h2>Classes</h2>\n");
            for class in &module.classes {
                html.push_str(&self.generate_html_class(class)?);
            }
        }

        html.push_str("    </div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }

    fn generate_html_function(&self, function: &DocFunction) -> Result<String> {
        let mut html = String::new();

        html.push_str("        <div class=\"function\">\n");
        html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", function.name, function.name));
        html.push_str(&format!("            <code class=\"signature\">{}</code>\n", function.signature));
        html.push_str(&format!("            <p class=\"description\">{}</p>\n", function.description));

        if !function.parameters.is_empty() {
            html.push_str("            <h4>Parameters</h4>\n");
            html.push_str("            <ul class=\"parameters\">\n");
            for param in &function.parameters {
                html.push_str(&format!(
                    "                <li><strong>{}</strong>{}: {}</li>\n",
                    param.name,
                    param.param_type.as_ref().map(|t| format!(" ({})", t)).unwrap_or_default(),
                    param.description
                ));
            }
            html.push_str("            </ul>\n");
        }

        if let Some(return_desc) = &function.return_description {
            html.push_str("            <h4>Returns</h4>\n");
            html.push_str(&format!("            <p>{}</p>\n", return_desc));
        }

        html.push_str("        </div>\n");

        Ok(html)
    }

    fn generate_html_class(&self, class: &DocClass) -> Result<String> {
        let mut html = String::new();

        html.push_str("        <div class=\"class\">\n");
        html.push_str(&format!("            <h3 id=\"{}\">{}</h3>\n", class.name, class.name));
        html.push_str(&format!("            <p class=\"description\">{}</p>\n", class.description));
        html.push_str("        </div>\n");

        Ok(html)
    }

    fn generate_markdown(&self, modules: &[DocModule], output_dir: &Path) -> Result<()> {
        // Generate README.md
        let readme_content = self.generate_markdown_index(modules)?;
        std::fs::write(output_dir.join("README.md"), readme_content)?;

        // Generate individual module pages
        for module in modules {
            let module_content = self.generate_markdown_module(module)?;
            let filename = format!("{}.md", module.name);
            std::fs::write(output_dir.join(filename), module_content)?;
        }

        Ok(())
    }

    fn generate_markdown_index(&self, modules: &[DocModule]) -> Result<String> {
        let mut md = String::new();

        md.push_str("# Nagari Documentation\n\n");
        md.push_str("## Modules\n\n");

        for module in modules {
            md.push_str(&format!("### [{}]({}.md)\n\n", module.name, module.name));
            md.push_str(&format!("{}\n\n", module.description));
            md.push_str(&format!(
                "- {} functions\n- {} classes\n\n",
                module.functions.len(),
                module.classes.len()
            ));
        }

        Ok(md)
    }

    fn generate_markdown_module(&self, module: &DocModule) -> Result<String> {
        let mut md = String::new();

        md.push_str(&format!("# {}\n\n", module.name));
        md.push_str(&format!("{}\n\n", module.description));

        if !module.functions.is_empty() {
            md.push_str("## Functions\n\n");
            for function in &module.functions {
                md.push_str(&self.generate_markdown_function(function)?);
            }
        }

        if !module.classes.is_empty() {
            md.push_str("## Classes\n\n");
            for class in &module.classes {
                md.push_str(&self.generate_markdown_class(class)?);
            }
        }

        Ok(md)
    }

    fn generate_markdown_function(&self, function: &DocFunction) -> Result<String> {
        let mut md = String::new();

        md.push_str(&format!("### {}\n\n", function.name));
        md.push_str(&format!("```nagari\n{}\n```\n\n", function.signature));
        md.push_str(&format!("{}\n\n", function.description));

        if !function.parameters.is_empty() {
            md.push_str("**Parameters:**\n\n");
            for param in &function.parameters {
                md.push_str(&format!(
                    "- `{}` {}: {}\n",
                    param.name,
                    param.param_type.as_ref().map(|t| format!("({})", t)).unwrap_or_default(),
                    param.description
                ));
            }
            md.push_str("\n");
        }

        if let Some(return_desc) = &function.return_description {
            md.push_str(&format!("**Returns:** {}\n\n", return_desc));
        }

        Ok(md)
    }

    fn generate_markdown_class(&self, class: &DocClass) -> Result<String> {
        let mut md = String::new();

        md.push_str(&format!("### {}\n\n", class.name));
        md.push_str(&format!("{}\n\n", class.description));

        Ok(md)
    }    fn generate_json(&self, modules: &[DocModule], output_dir: &Path) -> Result<()> {
        let json_content = serde_json::to_string_pretty(modules)?;
        std::fs::write(output_dir.join("documentation.json"), json_content)?;

        Ok(())
    }

    // Helper methods for property extraction
    fn extract_property_name(&self, line: &str) -> String {
        // Extract property name from a line
        if let Some(colon_pos) = line.find(':') {
            line[..colon_pos].trim().to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn extract_property_type(&self, line: &str) -> String {
        // Extract type annotation from a property line
        if let Some(colon_pos) = line.find(':') {
            let type_part = &line[colon_pos + 1..];
            if let Some(equals_pos) = type_part.find('=') {
                type_part[..equals_pos].trim().to_string()
            } else {
                type_part.trim().to_string()
            }
        } else {
            "Any".to_string()
        }
    }

    fn extract_property_description(&self, lines: &[String], start_index: usize) -> Option<String> {
        // Look for docstring or comment after the property
        for i in (start_index + 1)..(start_index + 3).min(lines.len()) {
            let line = lines[i].trim();
            if line.starts_with('#') {
                return Some(line.trim_start_matches('#').trim().to_string());
            } else if line.starts_with("\"\"\"") || line.starts_with("'''") {
                return Some(line.trim_start_matches("\"\"\"").trim_start_matches("'''").trim().to_string());
            }
        }
        None
    }

    fn infer_type_from_assignment(&self, assignment: &str) -> String {
        // Infer type from assignment value
        let assignment = assignment.trim();
        if assignment.starts_with('"') || assignment.starts_with('\'') {
            "str".to_string()
        } else if assignment == "True" || assignment == "False" {
            "bool".to_string()
        } else if assignment.chars().all(|c| c.is_ascii_digit()) {
            "int".to_string()
        } else if assignment.chars().any(|c| c == '.') && assignment.chars().filter(|c| !c.is_ascii_digit() && *c != '.').count() == 0 {
            "float".to_string()
        } else if assignment.starts_with('[') {
            "list".to_string()
        } else if assignment.starts_with('{') {
            "dict".to_string()
        } else {
            "Any".to_string()
        }
    }
}

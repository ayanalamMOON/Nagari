use crate::repl_engine::{ReplEngine, ReplValue};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct BuiltinCommands {
    commands: std::collections::HashMap<String, CommandInfo>,
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub aliases: Vec<String>,
}

pub trait ReplCommand {
    async fn execute(&self, args: &[&str], repl: &mut ReplEngine) -> Result<String>;
    fn get_help(&self) -> String;
}

impl BuiltinCommands {
    pub fn new() -> Self {
        let mut commands = std::collections::HashMap::new();

        // Add all builtin commands
        commands.insert("help".to_string(), CommandInfo {
            name: "help".to_string(),
            description: "Show help information".to_string(),
            usage: ".help [command]".to_string(),
            aliases: vec!["h".to_string(), "?".to_string()],
        });

        commands.insert("exit".to_string(), CommandInfo {
            name: "exit".to_string(),
            description: "Exit the REPL".to_string(),
            usage: ".exit".to_string(),
            aliases: vec!["quit".to_string(), "q".to_string()],
        });

        commands.insert("clear".to_string(), CommandInfo {
            name: "clear".to_string(),
            description: "Clear the screen".to_string(),
            usage: ".clear".to_string(),
            aliases: vec!["cls".to_string()],
        });

        commands.insert("history".to_string(), CommandInfo {
            name: "history".to_string(),
            description: "Show command history".to_string(),
            usage: ".history [count]".to_string(),
            aliases: vec!["hist".to_string()],
        });

        commands.insert("vars".to_string(), CommandInfo {
            name: "vars".to_string(),
            description: "Show current variables".to_string(),
            usage: ".vars".to_string(),
            aliases: vec!["variables".to_string()],
        });

        commands.insert("funcs".to_string(), CommandInfo {
            name: "funcs".to_string(),
            description: "Show current functions".to_string(),
            usage: ".funcs".to_string(),
            aliases: vec!["functions".to_string()],
        });

        commands.insert("reset".to_string(), CommandInfo {
            name: "reset".to_string(),
            description: "Reset the REPL context".to_string(),
            usage: ".reset".to_string(),
            aliases: vec!["restart".to_string()],
        });

        commands.insert("load".to_string(), CommandInfo {
            name: "load".to_string(),
            description: "Load a Nagari script file".to_string(),
            usage: ".load <file>".to_string(),
            aliases: vec!["source".to_string()],
        });

        commands.insert("save".to_string(), CommandInfo {
            name: "save".to_string(),
            description: "Save current session".to_string(),
            usage: ".save <file>".to_string(),
            aliases: vec![],
        });

        Self { commands }
    }

    pub async fn execute(&self, command: &str, args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        match command {
            "help" | "h" | "?" => self.help_command(args, repl).await,
            "exit" | "quit" | "q" => self.exit_command(args, repl).await,
            "clear" | "cls" => self.clear_command(args, repl).await,
            "history" | "hist" => self.history_command(args, repl).await,
            "vars" | "variables" => self.vars_command(args, repl).await,
            "funcs" | "functions" => self.funcs_command(args, repl).await,
            "reset" | "restart" => self.reset_command(args, repl).await,
            "load" | "source" => self.load_command(args, repl).await,
            "save" => self.save_command(args, repl).await,
            _ => Ok(format!("Unknown command: {}. Type .help for available commands.", command)),
        }
    }

    async fn help_command(&self, args: &[&str], _repl: &mut ReplEngine) -> Result<String> {
        if args.is_empty() {
            let mut output = String::from("Available commands:\n");

            for (_, info) in &self.commands {
                output.push_str(&format!("  {:<15} - {}\n", info.usage, info.description));
            }

            output.push_str("\nType .help <command> for detailed information about a specific command.\n");
            Ok(output)
        } else {
            let cmd_name = args[0];
            if let Some(info) = self.commands.get(cmd_name) {
                let mut output = format!("Command: {}\n", info.name);
                output.push_str(&format!("Description: {}\n", info.description));
                output.push_str(&format!("Usage: {}\n", info.usage));

                if !info.aliases.is_empty() {
                    output.push_str(&format!("Aliases: {}\n", info.aliases.join(", ")));
                }

                Ok(output)
            } else {
                Ok(format!("Unknown command: {}", cmd_name))
            }
        }
    }

    async fn exit_command(&self, _args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        repl.exit();
        Ok("Goodbye!".to_string())
    }

    async fn clear_command(&self, _args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        repl.clear_screen();
        Ok(String::new())
    }

    async fn history_command(&self, args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        let count = if args.is_empty() {
            None
        } else {
            Some(args[0].parse::<usize>().unwrap_or(10))
        };

        repl.show_history(count);
        Ok(String::new())
    }

    async fn vars_command(&self, _args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        let context = repl.get_context();
        let variables = context.list_variables();

        if variables.is_empty() {
            Ok("No variables defined.".to_string())
        } else {
            let mut output = String::from("Current variables:\n");

            for var in variables {
                let type_info = match &var.value {
                    ReplValue::Number(_) => "number",
                    ReplValue::String(_) => "string",
                    ReplValue::Boolean(_) => "boolean",
                    ReplValue::List(_) => "list",
                    ReplValue::Object(_) => "object",
                    ReplValue::Function(_) => "function",
                    ReplValue::Null => "null",
                    ReplValue::Undefined => "undefined",
                };

                let mutability = if var.mutable { "mut" } else { "const" };
                output.push_str(&format!("  {} {} : {} = {:?}\n", mutability, var.name, type_info, var.value));
            }

            Ok(output)
        }
    }

    async fn funcs_command(&self, _args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        let context = repl.get_context();
        let functions = context.list_functions();

        if functions.is_empty() {
            Ok("No functions defined.".to_string())
        } else {
            let mut output = String::from("Current functions:\n");

            for func in functions {
                let params: Vec<String> = func.parameters.iter()
                    .map(|p| {
                        if let Some(ref param_type) = p.param_type {
                            format!("{}: {}", p.name, param_type)
                        } else {
                            p.name.clone()
                        }
                    })
                    .collect();

                let return_type = func.return_type.as_deref().unwrap_or("any");
                output.push_str(&format!("  fn {}({}) -> {}\n", func.name, params.join(", "), return_type));
            }

            Ok(output)
        }
    }

    async fn reset_command(&self, _args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        repl.get_context_mut().reset();
        Ok("REPL context has been reset.".to_string())
    }

    async fn load_command(&self, args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        if args.is_empty() {
            return Ok("Usage: .load <file>".to_string());
        }

        let file_path = std::path::PathBuf::from(args[0]);

        match repl.load_script(&file_path).await {
            Ok(()) => Ok(format!("Successfully loaded: {}", file_path.display())),
            Err(e) => Ok(format!("Error loading file: {}", e)),
        }
    }

    async fn save_command(&self, args: &[&str], repl: &mut ReplEngine) -> Result<String> {
        if args.is_empty() {
            return Ok("Usage: .save <file>".to_string());
        }

        let file_path = std::path::PathBuf::from(args[0]);

        match repl.save_session(&file_path) {
            Ok(()) => Ok(format!("Session saved to: {}", file_path.display())),
            Err(e) => Ok(format!("Error saving session: {}", e)),
        }
    }

    pub fn get_command_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        for (name, info) in &self.commands {
            names.push(name.clone());
            names.extend(info.aliases.clone());
        }

        names.sort();
        names
    }

    pub fn get_command_info(&self, name: &str) -> Option<&CommandInfo> {
        // Check direct name first
        if let Some(info) = self.commands.get(name) {
            return Some(info);
        }

        // Check aliases
        for (_, info) in &self.commands {
            if info.aliases.contains(&name.to_string()) {
                return Some(info);
            }
        }

        None
    }
}

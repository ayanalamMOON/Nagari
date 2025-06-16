use reedline::{Reedline, Signal, DefaultPrompt, Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus};
use crossterm::style::{Color, Attribute};
use anyhow::Result;

use crate::repl_engine::{CodeCompleter, SyntaxHighlighter, ReplConfig};

pub struct ReplEditor {
    line_editor: Reedline,
    prompt: Box<dyn Prompt>,
}

pub struct NagariPrompt {
    prompt_text: String,
    continuation_text: String,
}

impl ReplEditor {
    pub fn new(config: &ReplConfig) -> Result<Self> {
        let mut line_editor = Reedline::create();

        // Configure history
        if config.history_size > 0 {
            line_editor = line_editor.with_history_session_id(Some("nagari-repl".to_string()));
        }

        let prompt = Box::new(NagariPrompt::new(
            config.prompt.clone(),
            config.continuation_prompt.clone(),
        ));

        Ok(Self {
            line_editor,
            prompt,
        })
    }

    pub async fn read_line(
        &mut self,
        prompt_text: &str,
        completer: &mut CodeCompleter,
        highlighter: &mut SyntaxHighlighter,
    ) -> Result<String> {
        // Update prompt text
        if let Some(nagari_prompt) = self.prompt.as_any().downcast_mut::<NagariPrompt>() {
            nagari_prompt.set_prompt(prompt_text.to_string());
        }

        // Set up completer and highlighter
        if completer.is_enabled() {
            self.line_editor = self.line_editor.with_completer(Box::new(completer.clone()));
        }

        if highlighter.is_enabled() {
            self.line_editor = self.line_editor.with_highlighter(Box::new(highlighter.clone()));
        }

        match self.line_editor.read_line(&*self.prompt) {
            Ok(Signal::Success(buffer)) => Ok(buffer),
            Ok(Signal::CtrlD) => Ok(".exit".to_string()),
            Ok(Signal::CtrlC) => Ok(String::new()),
            Err(e) => Err(anyhow::anyhow!("Input error: {}", e)),
        }
    }

    pub fn add_history(&mut self, line: String) {
        // Add line to history
        self.line_editor.add_history_entry(line);
    }

    pub fn set_completer(&mut self, completer: Box<dyn reedline::Completer>) {
        self.line_editor = self.line_editor.with_completer(completer);
    }

    pub fn set_highlighter(&mut self, highlighter: Box<dyn reedline::Highlighter>) {
        self.line_editor = self.line_editor.with_highlighter(highlighter);
    }

    pub fn set_validator(&mut self, validator: Box<dyn reedline::Validator>) {
        self.line_editor = self.line_editor.with_validator(validator);
    }
}

impl NagariPrompt {
    pub fn new(prompt: String, continuation: String) -> Self {
        Self {
            prompt_text: prompt,
            continuation_text: continuation,
        }
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt_text = prompt;
    }

    pub fn set_continuation(&mut self, continuation: String) {
        self.continuation_text = continuation;
    }
}

impl Prompt for NagariPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        self.prompt_text.as_str().into()
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        "".into()
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> std::borrow::Cow<str> {
        match edit_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => "".into(),
            PromptEditMode::Vi(vi_mode) => {
                match vi_mode {
                    reedline::PromptViMode::Normal => "[N]".into(),
                    reedline::PromptViMode::Insert => "[I]".into(),
                }
            }
        }
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        self.continuation_text.as_str().into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ).into()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct EditorConfig {
    pub vi_mode: bool,
    pub auto_pairs: bool,
    pub bracket_matching: bool,
    pub indent_size: usize,
    pub tab_size: usize,
    pub word_wrap: bool,
    pub show_line_numbers: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            vi_mode: false,
            auto_pairs: true,
            bracket_matching: true,
            indent_size: 4,
            tab_size: 4,
            word_wrap: true,
            show_line_numbers: false,
        }
    }
}

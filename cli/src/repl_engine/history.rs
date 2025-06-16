use chrono::{DateTime, Utc};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CommandHistory {
    commands: VecDeque<HistoryEntry>,
    max_size: usize,
    current_index: usize,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub execution_time: Option<std::time::Duration>,
    pub success: bool,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: VecDeque::new(),
            max_size,
            current_index: 0,
        }
    }

    pub fn add_command(&mut self, command: String) {
        let entry = HistoryEntry {
            command,
            timestamp: Utc::now(),
            execution_time: None,
            success: true,
        };

        self.commands.push_back(entry);

        // Remove oldest entries if we exceed max size
        while self.commands.len() > self.max_size {
            self.commands.pop_front();
        }

        self.current_index = self.commands.len();
    }

    pub fn add_command_with_result(
        &mut self,
        command: String,
        execution_time: std::time::Duration,
        success: bool,
    ) {
        let entry = HistoryEntry {
            command,
            timestamp: Utc::now(),
            execution_time: Some(execution_time),
            success,
        };

        self.commands.push_back(entry);

        while self.commands.len() > self.max_size {
            self.commands.pop_front();
        }

        self.current_index = self.commands.len();
    }

    pub fn get_previous(&mut self) -> Option<&str> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.commands.get(self.current_index).map(|entry| entry.command.as_str())
        } else {
            None
        }
    }

    pub fn get_next(&mut self) -> Option<&str> {
        if self.current_index < self.commands.len() - 1 {
            self.current_index += 1;
            self.commands.get(self.current_index).map(|entry| entry.command.as_str())
        } else {
            self.current_index = self.commands.len();
            None
        }
    }

    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        self.commands
            .iter()
            .filter(|entry| entry.command.contains(query))
            .collect()
    }

    pub fn get_recent(&self, count: usize) -> Vec<&HistoryEntry> {
        self.commands
            .iter()
            .rev()
            .take(count)
            .collect()
    }

    pub fn show(&self, count: Option<usize>) {
        let entries = match count {
            Some(n) => self.get_recent(n),
            None => self.commands.iter().collect(),
        };

        for (i, entry) in entries.iter().rev().enumerate() {
            let success_indicator = if entry.success { "✓" } else { "✗" };
            let time_str = entry.timestamp.format("%H:%M:%S");

            if let Some(exec_time) = entry.execution_time {
                println!(
                    "{:3}: {} [{}] ({:.2}ms) {}",
                    self.commands.len() - entries.len() + i + 1,
                    success_indicator,
                    time_str,
                    exec_time.as_millis(),
                    entry.command
                );
            } else {
                println!(
                    "{:3}: {} [{}] {}",
                    self.commands.len() - entries.len() + i + 1,
                    success_indicator,
                    time_str,
                    entry.command
                );
            }
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.current_index = 0;
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn get_statistics(&self) -> HistoryStats {
        let total_commands = self.commands.len();
        let successful_commands = self.commands.iter().filter(|e| e.success).count();
        let failed_commands = total_commands - successful_commands;

        let total_execution_time: std::time::Duration = self.commands
            .iter()
            .filter_map(|e| e.execution_time)
            .sum();

        let average_execution_time = if total_commands > 0 {
            total_execution_time / total_commands as u32
        } else {
            std::time::Duration::from_millis(0)
        };

        HistoryStats {
            total_commands,
            successful_commands,
            failed_commands,
            total_execution_time,
            average_execution_time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_commands: usize,
    pub successful_commands: usize,
    pub failed_commands: usize,
    pub total_execution_time: std::time::Duration,
    pub average_execution_time: std::time::Duration,
}

impl std::fmt::Display for HistoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Command History Statistics:")?;
        writeln!(f, "  Total commands: {}", self.total_commands)?;
        writeln!(f, "  Successful: {}", self.successful_commands)?;
        writeln!(f, "  Failed: {}", self.failed_commands)?;
        writeln!(f, "  Success rate: {:.1}%",
                 if self.total_commands > 0 {
                     (self.successful_commands as f64 / self.total_commands as f64) * 100.0
                 } else {
                     0.0
                 })?;
        writeln!(f, "  Total execution time: {:.2}s", self.total_execution_time.as_secs_f64())?;
        writeln!(f, "  Average execution time: {:.2}ms", self.average_execution_time.as_millis())
    }
}

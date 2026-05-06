use crate::modules::{ExecutionContext, ModuleCommand};

pub const NAME: &str = "shell";

fn execute(args: &[&str], context: &ExecutionContext) -> String {
    if args.is_empty() {
        return "Usage: shell.execute <command...>".to_string();
    }
    let _ = context;

    #[cfg(target_os = "ios")]
    {
        return "shell.execute is not available on iOS".to_string();
    }

    #[cfg(not(target_os = "ios"))]
    {
        use std::process::Command;

        let command_line = args.join(" ");
        let output = {
            #[cfg(target_os = "windows")]
            {
                Command::new("cmd").args(["/C", &command_line]).output()
            }
            #[cfg(not(target_os = "windows"))]
            {
                Command::new("sh").args(["-c", &command_line]).output()
            }
        };

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let mut lines = Vec::new();
                lines.push(format!("exit: {:?}", output.status.code()));
                if !stdout.is_empty() {
                    lines.push(format!("stdout:\n{stdout}"));
                }
                if !stderr.is_empty() {
                    lines.push(format!("stderr:\n{stderr}"));
                }
                lines.join("\n")
            }
            Err(err) => format!("Failed to execute shell command: {err}"),
        }
    }
}

pub fn commands() -> &'static [ModuleCommand] {
    &[ModuleCommand {
        name: "execute",
        description: "execute shell command(s): shell.execute <command...>",
        run: execute,
    }]
}

use clap::Command;

pub fn create_history_command() -> Command {
    Command::new("history")
        .about("Undo & Replay support for target project")
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(create_history_undo_command())
        .subcommand(create_history_replay_command())
}

fn create_history_undo_command() -> Command {
    Command::new("undo")
        .about("Undo last operation")
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
}

fn create_history_replay_command() -> Command {
    Command::new("replay")
        .about("Replay recorder history")
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
}

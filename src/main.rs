use std::fmt::Display;
use std::process::Command;

use clap::Parser;
use hyprland::data::{Monitor, Workspace, Workspaces};
use hyprland::event_listener::EventListener;
use hyprland::prelude::*;

use self::cli::{Cli, Command as CliCommand, WorkspaceCommand};

mod cli;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::Daemon => daemon(),
        CliCommand::Workspace {
            default_spaces,
            command,
        } => match command {
            None => print!("{}", workspaces(default_spaces)),
            Some(WorkspaceCommand::Update { default_spaces }) => {
                eww_workspace_update(default_spaces)
            }
        },
    }
}

fn daemon() {
    let mut listener = EventListener::new();
    listener.add_workspace_change_handler(|_| eww_workspace_update(5));
    listener.start_listener().unwrap();
}

fn workspaces(default_spaces: usize) -> String {
    let workspaces = Workspaces::get().unwrap();
    EwwWorkspaces::new(workspaces, default_spaces).to_string()
}

fn eww_workspace_update(default_spaces: usize) {
    let mut command = Command::new("eww");
    command.args([
        "update",
        &format!("workspace={}", workspaces(default_spaces)),
    ]);
    let _ = command.spawn();
}

struct EwwWorkspaces(Vec<EwwWorkspaceButton>);

impl EwwWorkspaces {
    fn new(workspaces: Workspaces, default_spaces: usize) -> Self {
        let mut buttons = workspaces
            .iter()
            .map(|x| x.into())
            .collect::<Vec<EwwWorkspaceButton>>();
        buttons.sort_by(|a, b| a.id.cmp(&b.id));
        let mut results = Vec::new();

        for i in 0..(default_spaces) {
            if let Ok(i) = buttons.binary_search_by_key(&((i + 1) as i32), |x| x.id) {
                results.push(buttons[i].clone());
            } else {
                results.push(EwwWorkspaceButton::empty(i as i32 + 1))
            }
        }
        Self(results)
    }
}

#[derive(Clone)]
struct EwwWorkspaceButton {
    id: i32,
    state: EwwWorkspaceButtonState,
}

#[derive(Clone)]
enum EwwWorkspaceButtonState {
    Emtpy,
    Active,
    Contains,
}

impl EwwWorkspaceButton {
    fn empty(id: i32) -> Self {
        Self {
            id,
            state: EwwWorkspaceButtonState::Emtpy,
        }
    }
}

fn focus_workspace(workspace_id: i32) -> String {
    format!(
        "hyprctl dispatch workspace {};/home/munske/code/eww_api/target/release/eww_api workspace update",
        workspace_id,
    )
}

impl From<&Workspace> for EwwWorkspaceButton {
    fn from(value: &Workspace) -> Self {
        let mut state = if value.windows == 0 {
            EwwWorkspaceButtonState::Emtpy
        } else {
            EwwWorkspaceButtonState::Contains
        };
        if Monitor::get_active().unwrap().active_workspace.id == value.id {
            state = EwwWorkspaceButtonState::Active;
        };
        EwwWorkspaceButton {
            id: value.id,
            state,
        }
    }
}

impl Display for EwwWorkspaceButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(button :onclick \"{}\" :class \"workspace\" \"{}\")",
            focus_workspace(self.id),
            match self.state {
                EwwWorkspaceButtonState::Emtpy => "",
                EwwWorkspaceButtonState::Active => "",
                EwwWorkspaceButtonState::Contains => "",
            }
        )
    }
}

impl Display for EwwWorkspaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(box :class \"workspaces\" :orientation \"h\" :space-evenly \"false\" {} )",
            self.0
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
//     :orientation \"h\"
//     :spacing 5
//     :space-evenly \"false\"
//     (button :onclick \"bspc desktop -f $ws1\"
//             :class	\"$un$o1$f1\"
//             \"$ic_1\"
//     )
//     (button :onclick \"bspc desktop -f $ws2\"
//             :class \"$un$o2$f2\"
//             \"$ic_2\")
//     (button :onclick \"bspc desktop -f $ws3\"	:class \"$un$o3$f3\" \"$ic_3\")
//     (button :onclick \"bspc desktop -f $ws4\"	:class \"$un$o4$f4\"	\"$ic_4\")
//     (button :onclick \"bspc desktop -f $ws5\"	:class \"$un$o5$f5\" \"$ic_5\")
//     (button :onclick \"bspc desktop -f $ws6\"	:class \"$un$o6$f6\" \"$ic_6\")
// )"

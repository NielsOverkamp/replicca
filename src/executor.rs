use websocket::WebSocketResult;

use crate::turtle::TurtleState;
use crate::turtle_websocket::{Command, UpEvent, TurtleConnection, TaskCommand};
use json::JsonValue;

pub enum Task {
    Fell, FirstTree, Anon(String)
}

impl Task {
    pub fn code(&self) -> &str {
        match self {
            Task::Fell => "fell",
            Task::FirstTree => "first_tree",
            Task::Anon(s) => s.as_str()
        }
    }

    pub fn from_code(s: &str) -> Self {
        match s {
            "fell" => Task::Fell,
            "first_tree" => Task::FirstTree,
            _ => Task::Anon(s.to_lowercase()),
        }
    }
}

pub struct TaskExecutor {
    pub turtle: TurtleState,
    pub connection: TurtleConnection,
}

impl TaskExecutor {
    pub fn new(turtle: TurtleState, connection: TurtleConnection) -> TaskExecutor {
        Self { turtle, connection }
    }

    // TODO deal with websocket errors better and maybe even internally
    pub fn execute<F>(&mut self, task: Task, event_handler: F) -> WebSocketResult<()>
        where F: Fn(UpEvent, &mut TaskExecutor) -> bool {
        let command = match task {
            Task::Anon(_) => Command::AnonTask(task.code().to_string()),
            _ => Command::Task(task.code().to_string())
        };
        let id = self.connection.send_command(command)?;
        let mut continue_execution = true;
        while continue_execution {
            let event = self.connection.receive_event();
            if let Err(e) = event {
                eprintln!("Got unknown event or whatever: {}", e);
                continue;
            }
            let event = event.unwrap();
            continue_execution = match event {
                UpEvent::TaskFinish => {
                    self.connection.send_task_command(TaskCommand::FinishResponse);
                    false
                },
                UpEvent::StateUpdate(_) | UpEvent::PositionUpdate(_) | UpEvent::InventoryUpdate(_) => {
                    self.handle_update_event(event);
                    true
                }
                UpEvent::TaskError(_) => {
                    let continue_execution = event_handler(event, self);
                    self.connection.send_task_command(TaskCommand::ErrorResponse(continue_execution));
                    continue_execution
                },
                event => {
                    if !event_handler(event, self) {
                        self.connection.send_task_command(TaskCommand::Cancel);
                        false
                    } else {
                        true
                    }
                }
            };
        };
        Ok(())
    }

    pub fn handle_update_event(&mut self, event: UpEvent) {
        match event {
            UpEvent::StateUpdate(s) => {
                self.turtle = s;
                println!("Updated turtle state: {:?}",self.turtle);
            },
            UpEvent::PositionUpdate(p) => {
                self.turtle.position = p;
                println!("Updated turtle position: {:?}", self.turtle.position);
            },
            UpEvent::InventoryUpdate(di) => {
                di.apply(&mut self.turtle.inventory);
                println!("Updated turtle inventory: {:?}", self.turtle.inventory);
            },
            _ => {},
        }
    }

    pub fn default_event_handler(event: UpEvent, exec: &mut Self) -> bool {
        match event {
            UpEvent::TaskError(e) => false,
            UpEvent::TaskCancelled => false,
            UpEvent::Error => false,
            _ => {
                println!("unexpected event in default event handler: {:?}", event);
                true
            }
        }
    }
}

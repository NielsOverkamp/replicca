use websocket::WebSocketResult;

use crate::turtle::TurtleState;
use crate::turtle_websocket::{Command, UpEvent, TurtleConnection, TaskCommand, ReceiveError};
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

impl Into<JsonValue> for &Task {
    fn into(self) -> JsonValue {
        json::object! {
            n: self.code(),
            a: match self {
                Task::RegrowFirstTree(slot) => JsonValue::from(*slot),
                _ => JsonValue::Null,
            }
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
    pub fn execute<E, Q>(&mut self, task: Task, event_handler: E, question_handler: Q) -> WebSocketResult<()>
        where E: Fn(UpEvent, &mut TaskExecutor) -> bool,
                Q: Fn(String, &mut TaskExecutor) -> JsonValue {
        let command = match task {
            Task::Anon(_) => Command::AnonTask(JsonValue::from(task.code())),
            _ => Command::Task(task)
        };
        let id = self.connection.send_command(command)?;
        let mut continue_execution = true;
        while continue_execution {
            let event = self.connection.receive_event();
            if let Err(e) = event {
                match e {
                    ReceiveError::WebsocketError(e) => return Err(e),
                    ReceiveError::MessageError(e) => eprintln!("Got unexpected message: {}", e)
                }
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
                UpEvent::TaskQuestion(q) => {
                    let answer = question_handler(q, self);
                    self.connection.send_task_command(TaskCommand::QuestionResponse(answer));
                    true
                }
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

    pub fn default_event_handler(event: UpEvent, _: &mut Self) -> bool {
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

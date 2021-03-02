use std::{env, thread};
use std::error::Error;
use std::num::Wrapping;
use std::sync::mpsc;
use std::thread::JoinHandle;

use json::JsonValue;
use websocket::{Message, OwnedMessage, WebSocketResult};
use websocket::server::NoTlsAcceptor;
use websocket::stream::sync::TcpStream;
use websocket::sync::{Client, Server};

use crate::turtle::{Inventory, DeltaInventory, Position, TurtleState};

pub enum Command {
    Eval(String),
    AnonTask(String),
    Task(String),
    Move(String),
}

impl Command {
    pub fn code(&self) -> &'static str {
        match self {
            Command::Eval(_) => "EVAL",
            Command::AnonTask(_) | Command::Task(_) => "TASK",
            Command::Move(_) => "MOVE"
        }
    }

    pub fn into_json(self, id: u32) -> String {
        let code = self.code();
        match self {
            Command::AnonTask(s) | Command::Task(s) | Command::Eval(s) | Command::Move(s) => {
                json::stringify(json::object! {
                    c: code,
                    b: s,
                    id: id
                })
            }
        }
    }
}

#[derive(Debug)]
pub enum UpEvent {
    TaskError(TaskError),
    EvalResponse(JsonValue),
    MoveResponse(Result<(), (String, usize)>),
    TaskFinish,
    TaskCancelled,
    StateUpdate(TurtleState),
    PositionUpdate(Position),
    InventoryUpdate(DeltaInventory),
    Error,
}

impl From<&JsonValue> for UpEvent {
    fn from(jv: &JsonValue) -> Self {
        if let JsonValue::Object(o) = jv {
            if let Some(code) = o["c"].as_str() {
                match code {
                    "task_error" => UpEvent::TaskError(TaskError::from(&o["b"])),
                    "eval_response" => UpEvent::EvalResponse(o["b"].clone()),
                    "move_response" => {
                        if jv.has_key("b") {
                            UpEvent::MoveResponse(Err((o["b"]["e"].as_str().unwrap().to_owned(), o["b"]["c"].as_usize().unwrap())))
                        } else {
                            UpEvent::MoveResponse(Ok(()))
                        }
                    },
                    "task_finish" => UpEvent::TaskFinish,
                    "task_cancelled" => UpEvent::TaskCancelled,
                    "state_update" => UpEvent::StateUpdate(TurtleState::from(&o["b"])),
                    "position_update" => UpEvent::PositionUpdate(Position::from(&o["b"])),
                    "inventory_update" => UpEvent::InventoryUpdate(DeltaInventory::from(&o["b"])),
                    "error" => UpEvent::Error,
                    _ => panic!("Unknown event code {}", code)
                }
            } else {
                panic!("Expect event code string, got {}", o["c"])
            }
        } else {
            panic!("Expected json object, got {}", jv)
        }
    }
}

#[derive(Debug)]
pub enum TaskCommand {
    FinishResponse,
    Cancel,
    ErrorResponse(bool),
}


impl Into<JsonValue> for &TaskCommand {
    fn into(self) -> JsonValue {
        match self {
            TaskCommand::FinishResponse => json::object! {
                c: "task_finish_response"
            },
            TaskCommand::Cancel => json::object! {
                c: "task_cancel"
            },
            TaskCommand::ErrorResponse(b) => json::object! {
                c: "task_error_response",
                b: *b
            },
        }
    }
}

#[derive(Debug)]
pub enum TaskError {
    FuelLow,
    Obstacle,
}

impl TaskError {
    pub fn from_code(s: &str) -> Self {
        match s {
            "fuel" => Self::FuelLow,
            "obstacle" => Self::Obstacle,
            _ => panic!("Unknown task error code {}", s),
        }
    }
}

impl From<&JsonValue> for TaskError {
    fn from(jv: &JsonValue) -> Self {
        if jv.is_string() {
                Self::from_code(jv.as_str().unwrap())
        } else {
            panic!("Expected json string, got {}", jv)
        }
    }
}

pub fn spawn_websocket_listener() -> Result<(mpsc::Receiver<TurtleConnection>, JoinHandle<()>), Box<dyn std::error::Error>> {
    let address = env::var("ADDRESS").unwrap_or(String::from("localhost"));
    let port = env::var("PORT").unwrap_or(String::from("17576"));

    let server: Server<NoTlsAcceptor> = Server::bind(format!("{}:{}", address, port))?;

    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        for connection in server.filter_map(Result::ok) {
            let mut client: Client<TcpStream> = connection.accept().unwrap();
            tx.send(TurtleConnection::new(client));
        }
    });

    return Ok((rx, handle));
}

pub struct TurtleConnection {
    ws_client: Client<TcpStream>,
    last_id: Wrapping<u32>,
}


impl TurtleConnection {
    pub fn new(client: Client<TcpStream>) -> Self {
        Self {
            ws_client: client,
            last_id: Wrapping(0u32),
        }
    }

    pub fn send_command(&mut self, command: Command) -> WebSocketResult<u32> {
        let id = self.last_id.0;
        self.last_id += Wrapping(1u32);
        let _ = self.ws_client.send_message(&Message::text(command.into_json(id)))?;
        return Ok(id);
    }

    pub fn send_task_command(&mut self, command: TaskCommand) -> WebSocketResult<()> {
        self.ws_client.send_message(&Message::text(json::stringify(Into::<JsonValue>::into(&command))))
    }

    pub fn receive_event(&mut self) -> Result<UpEvent, Box<dyn Error>> {
        if let OwnedMessage::Text(s) = self.ws_client.recv_message()? {
            Ok(UpEvent::from(&json::parse(s.as_str()).unwrap()))
        } else {
            return Err("Got unexpected non text message".into());
        }
    }
}

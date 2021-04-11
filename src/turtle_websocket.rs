use std::{env, thread};
use std::error::Error;
use std::num::Wrapping;
use std::sync::mpsc;
use std::thread::JoinHandle;

use hyper::http::uri;
use json::JsonValue;
use websocket::{Message, OwnedMessage, WebSocketError, WebSocketResult};
use websocket::server::NoTlsAcceptor;
use websocket::server::sync::AcceptResult;
use websocket::stream::sync::TcpStream;
use websocket::sync::{Client, Server};
use websocket::sync::server::Upgrade;

use crate::executor::Task;
use crate::turtle::{DeltaInventory, Position, TurtleState};
use std::collections::HashMap;

pub enum Command {
    Eval(String),
    AnonTask(JsonValue),
    Task(Task),
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

    pub fn into_json(self) -> JsonValue {
        let code = self.code();
        json::object! {
                    c: code,
                    b: match self {
                        Command::Eval(s) | Command::Move(s) => JsonValue::from(s),
                        Command::AnonTask(jv) => jv,
                        Command::Task(t) => (&t).into(),
                    },
        }
    }
}

#[derive(Debug)]
pub enum UpEvent {
    TaskError(TaskError),
    TaskQuestion(String),
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
                    "task_question" => UpEvent::TaskQuestion(o["b"].as_str().unwrap().to_owned()),
                    "eval_response" => UpEvent::EvalResponse(o["b"].clone()),
                    "move_response" => {
                        if jv.has_key("b") {
                            UpEvent::MoveResponse(Err((o["b"]["e"].as_str().unwrap().to_owned(), o["b"]["c"].as_usize().unwrap())))
                        } else {
                            UpEvent::MoveResponse(Ok(()))
                        }
                    }
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
    QuestionResponse(JsonValue),
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
            TaskCommand::QuestionResponse(b) => json::object! {
                c: "task_answer",
                b: b.clone()
            }
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
        let mut reconnect_map: HashMap<u32, mpsc::Sender<Client<TcpStream>>> = HashMap::new();

        for connection in server.filter_map(Result::ok) {
            let connection: Upgrade<TcpStream> = connection;

            let req_uri = connection.request.subject.1.clone();
            let hyper_uri = req_uri.to_string().parse::<uri::Uri>().unwrap();
            let mut path_iter = hyper_uri.path().split("/");
            let id =
                path_iter.next().filter(|s| (*s).eq(""))
                    .and_then(|_| path_iter.next()).filter(|s| (*s).eq("ws"))
                    .and_then(|_| path_iter.next())
                    .and_then(|s| s.parse::<u32>().ok())
                    .ok_or(format!("Expected ws request on /ws/{{id}}, got {}", req_uri));
            if let Ok(id) = id {
                let client: Client<TcpStream> = connection.accept().unwrap();
                if reconnect_map.contains_key(&id) {
                    reconnect_map.get(&id).unwrap().send(client);
                } else {
                    let (recon_tx, recon_rx) = mpsc::channel();
                    reconnect_map.insert(id, recon_tx);
                    tx.send(TurtleConnection::new(client, recon_rx)).unwrap();
                }
            } else {
                eprintln!("{}", id.unwrap_err())
            }
        }
    });

    return Ok((rx, handle));
}

#[derive(Debug)]
pub enum ReceiveError {
    WebsocketError(WebSocketError),
    MessageError(String),
}

pub struct TurtleConnection {
    ws_client: Client<TcpStream>,
    reconnect_receiver: mpsc::Receiver<Client<TcpStream>>,
    last_id: Wrapping<u32>,
}


impl TurtleConnection {
    pub fn new(client: Client<TcpStream>, reconnect_receiver: mpsc::Receiver<Client<TcpStream>>) -> Self {
        Self {
            ws_client: client,
            reconnect_receiver,
            last_id: Wrapping(1u32),
        }
    }

    pub fn send(&mut self, message: String) {
        println!("Sending: {}", message);
        loop {
            match self.ws_client.send_message(&Message::text(&message)) {
                Ok(()) => return,
                Err(e) => {
                    eprintln!("Websocket receive got error: {}", e);
                    self.ws_client = self.reconnect_receiver.recv().unwrap();
                }
            }
        }

    }

    pub fn receive(&mut self) -> Result<String, ReceiveError> {
        loop {
            match self.ws_client.recv_message() {
                Ok(OwnedMessage::Text(m)) => {
                    println!("Received: {}", m);
                    return Ok(m)
                },
                Ok(_) => return Err(ReceiveError::MessageError("Got unexpected non text message".into())),
                Err(e) => {
                    eprintln!("Websocket receive got error: {}", e);
                    self.ws_client = self.reconnect_receiver.recv().unwrap();
                    println!("Reconnected");
                }
            }
        }
    }

    pub fn send_command(&mut self, command: Command) -> u32 {
        let mid = self.last_id.0;
        self.last_id += Wrapping(1u32);
        self.send(json::stringify(
            json::object! {
                mid: mid,
                cid: 0,
                c: "COMMAND",
                b: command.into_json()
            }));
        mid
    }

    pub fn send_task_command(&mut self, command: TaskCommand, cid: u32) -> u32 {
        let mid = self.last_id.0;
        self.last_id += Wrapping(1u32);

        self.send(json::stringify(
            json::object! {
                mid: mid,
                cid: cid,
                c: "TASK_EVENT",
                b: Into::<JsonValue>::into(&command)
            }));
        mid
    }

    pub fn receive_event(&mut self) -> Result<(UpEvent, u32, u32), ReceiveError> {
        let s = self.receive()?;
        let jv = json::parse(s.as_str())
            .map_err(|_| ReceiveError::MessageError(format!("Could not parse json string {}", s)))?;
        if let JsonValue::Object(_) = &jv {
            let mid: u32 = jv["mid"].as_u32().unwrap_or(0);
            let cid: u32 = jv["cid"].as_u32().unwrap_or(0);
            // TODO parse from Object i/o JsonValue
            return Ok((UpEvent::from(&jv), mid, cid));
        } else {
            return Err(ReceiveError::MessageError(format!("Expected json object, got {:?}", jv)));
        }
    }
}

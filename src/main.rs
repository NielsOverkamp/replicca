use std::sync::{Mutex, Arc};
use std::thread;
use crate::turtle::TurtleState;
use crate::executor::TaskExecutor;

mod turtle_websocket;
mod turtle_rest;
mod turtle;
mod executor;
mod maneuver;
mod console;

pub type TurtleList = Arc<Mutex<Vec<TaskExecutor>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client_rx, ws_thread_handle) = turtle_websocket::spawn_websocket_listener()?;

    let turtle_list = Arc::new(Mutex::new(Vec::new()));
    let turtle_list_handle;
    {
        let turtle_list = Arc::clone(&turtle_list);
        turtle_list_handle = thread::spawn(move || {
            loop {
                let connection = client_rx.recv().unwrap();
                let turtle = TurtleState::default();
                turtle_list.lock().unwrap().push(TaskExecutor::new(turtle, connection))
            }
        })
    }

    return console::spawn_console(turtle_list)?.join().map_err(|_| "thread failed".into());
}

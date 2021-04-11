use std::sync::{Mutex, Arc};
use std::thread;
use crate::turtle::TurtleState;
use crate::executor::TaskExecutor;
use crate::turtle_runner::Runner;
use std::error::Error;

mod turtle_websocket;
mod turtle_rest;
mod turtle;
mod executor;
mod maneuver;
mod console;
mod turtle_runner;

pub type TurtleList = Arc<Mutex<Vec<TaskExecutor>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client_rx, _) = turtle_websocket::spawn_websocket_listener()?;

    // let turtle_list = Arc::new(Mutex::new(Vec::new()));
    let handle;
    {
        // let turtle_list = Arc::clone(&turtle_list);
        handle = thread::spawn(move || {
            loop {
                let connection = client_rx.recv().unwrap();
                let turtle = TurtleState::default();
                let task_executor = TaskExecutor::new(turtle, connection);
                let mut runner = Runner {
                    executor: task_executor
                };
                thread::spawn(move || {
                    match runner.run() {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Runner encountered error: {:?}", e)
                        }
                    }
                    thread::park()
                });
                // turtle_list.lock().unwrap().push(task_executor)
            }
        })
    }

    // return console::spawn_console(turtle_list)?.join().map_err(|_| "thread failed".into());
    return handle.join().map_err(|_| "thread failed".into());
}

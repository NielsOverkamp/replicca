use std::str::{FromStr, SplitWhitespace};
use std::thread;
use std::thread::JoinHandle;
use std::io::{stdin, BufRead, stdout, Write};
use crate::TurtleList;
use crate::turtle_websocket::{Command, UpEvent};
use crate::executor::TaskExecutor;
use crate::executor::Task;


pub enum ConsoleCommand {
    Eval, Task, Move, Turtle, List, Exit
}

impl FromStr for ConsoleCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eval" => Ok(ConsoleCommand::Eval),
            "task" => Ok(ConsoleCommand::Task),
            "move" => Ok(ConsoleCommand::Move),
            "turtle" => Ok(ConsoleCommand::Turtle),
            "list" => Ok(ConsoleCommand::List),
            "exit" => Ok(ConsoleCommand::Exit),
            _ => Err(())
        }
    }
}

pub fn spawn_console(mut turtles: TurtleList) -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
    Ok(thread::spawn(move || {
        let mut stdin = stdin();
        let mut handle = stdin.lock();
        let mut selected = None;
        loop {
            print!("> ");
            stdout().flush();
            let mut input = String::new();
            match handle.read_line(&mut input) {
                Ok(0) => {
                    break;
                }
                Ok(_) => {
                    let mut input = input.split_whitespace();
                    if let Some(s) = input.next() {
                        match s.parse() {
                            Ok(ConsoleCommand::Exit) => {
                                break;
                            }
                            Ok(c) => {
                                match parse_command(c, input, &mut turtles, selected) {
                                    Ok(s) => selected = s,
                                    Err(e) => eprintln!("Error: {}", e),
                                }
                            },
                            Err(_) => eprintln!("Unknown command: {}", s),
                        }
                    }
                }
                Err(error) => eprintln!("Error: {}", error),
            }
        }
    }))
}

pub fn parse_command(command: ConsoleCommand, mut input: SplitWhitespace, turtles: &mut TurtleList, selected: Option<usize>) -> Result<Option<usize>, String> {
    match command {
        ConsoleCommand::Eval => {
            if selected.is_none() {
                return Err("No turtle selected".to_string())
            }

            let first = input.next();

            let mut body;
            if let Some(s) = first {
                body = String::from(s);
                input.for_each(|w| {
                    body.push_str(" ");
                    body.push_str(w);
                });
            } else {
                body = String::new();
            }

            let turtle_lock = turtles.lock();
            let mut turtle_lock = turtle_lock.unwrap();
            let exec: &mut TaskExecutor = turtle_lock.get_mut(selected.unwrap()).unwrap();

            exec.connection.send_command(Command::Eval(body)).map_err(|_| "Sending error".to_string())?;
            match exec.connection.receive_event().unwrap() {
                UpEvent::EvalResponse(jv) => println!("{}", jv),
                _ => {}
            }
            Ok(selected)

        }
        ConsoleCommand::Task => {
            let task = input.next().map(Task::from_code);
            if task.is_none() {
                return Err("Task requires 1 argument".to_string())
            }
            let turtle_lock = turtles.lock();
            let mut turtle_lock = turtle_lock.unwrap();
            let exec: &mut TaskExecutor = turtle_lock.get_mut(selected.unwrap()).unwrap();

            match task.unwrap() {
                Task::Anon(t) => Err("Anonymous tasks not supported".to_string()),
                e => exec.execute(e, TaskExecutor::default_event_handler)
                    .map_err(|e| format!("Error occurred at sending task: {}", e))
            }.map(|_| selected)
        }
        ConsoleCommand::Move => {
            let s: String = input.into_iter().collect();
            let turtle_lock = turtles.lock();
            let mut turtle_lock = turtle_lock.unwrap();
            let exec: &mut TaskExecutor = turtle_lock.get_mut(selected.unwrap()).unwrap();

            exec.connection.send_command(Command::Move(s.clone())).map_err(|_| "Sending error".to_string())?;
            match exec.connection.receive_event().unwrap() {
                UpEvent::MoveResponse(Ok(())) => {
                    println!("finished!");
                    Ok(selected)
                },
                UpEvent::MoveResponse(Err((err, completed))) => {
                    println!("Error during move: {}", err);
                    let mut chars = s.chars();
                    println!("completed:\t{}", chars.by_ref().take(completed-1).collect::<String>());
                    println!("not completed:\t{}", chars.collect::<String>());
                    Ok(selected)
                }
                r => Err(format!("Expected move response, got {:?}", r))
            }

        }
        ConsoleCommand::Turtle => {
            let s = match input.next() {
                None => Err("Turtle requires 1 argument".to_string()),
                Some(arg1) => arg1.parse().map_err(|_| "Expected an integer argument at position 1".to_string()),
            }?;
            let turtle_count = turtles.lock().unwrap().len();
            if s >= turtle_count {
                return Err(format!("Only {} turtles connected, {} is too high", turtle_count, s));
            } else {
                return Ok(Some(s));
            }
        }
        ConsoleCommand::List => {
            turtles.lock().unwrap().iter()
                .map(|t| &t.turtle.label)
                .zip(0..)
                .for_each(|(l, i)| println!("{}: {}", i, l));
            return Ok(selected)
        }
        _ => {Err("Not implemented".to_string())}
    }
}

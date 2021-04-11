use json::JsonValue;

use crate::executor::{Task, TaskExecutor};
use crate::turtle_websocket::{TaskError, UpEvent};
use std::error::Error;

pub struct Runner {
    pub executor: TaskExecutor,
}

impl Runner {
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let question_handler = |q: String, e: &mut TaskExecutor| {
            match q.as_str() {
                "replant" => {
                    if let Some((_, sapling_slot)) = e.turtle.inventory.find(|i| i.name.ends_with("sapling") && i.count > 0) {
                        json::array![
                            true,
                            true,
                            sapling_slot
                        ]
                    } else {
                        json::array![
                            false
                        ]
                    }
                }
                _ => JsonValue::from(false),
            }
        };
        if !self.executor.execute(Task::FirstTree, TaskExecutor::default_event_handler, question_handler)? {
            return Ok(())
        }

        let event_handler = |e: UpEvent, exc: &mut TaskExecutor| {
            match e {
                UpEvent::TaskError(e) => match e {
                    TaskError::FuelLow => {
                        if let Some((_, log_slot)) = exc.turtle.inventory.find(|i| i.name.ends_with("log") && i.count > 0) {
                            exc.execute(Task::RefuelLogs(log_slot as u8, 2), TaskExecutor::default_event_handler, TaskExecutor::null_question_handler).unwrap()
                        } else {
                            false
                        }
                    }
                    TaskError::Obstacle => false
                }
                _ => TaskExecutor::default_event_handler(e, exc)
            }
        };

        // Assume replant was successful, I don't know how to handle the other case anyways
        loop {
            let log_count: u16 = self.executor.turtle.inventory.find_all(|i| i.name.ends_with("log"))
                .map(|(i, _)| i.count as u16)
                .sum();
            println!("Collected {} logs", log_count);
            if log_count >= 16 {
                break;
            }

            if !self.executor.execute(Task::Fell, event_handler, question_handler)? {
                return Ok(())
            }
        };
        Ok(())
    }
}
use std::error::Error;

use json::JsonValue;

use crate::executor::{Task, TaskExecutor};

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
                            true
                        ]
                    }
                }
                _ => JsonValue::from(false),
            }
        };
        self.executor.execute(Task::FirstTree, TaskExecutor::default_event_handler, question_handler)?;
        // Assume replant was successful, I don't know how to handle the other case anyways
        loop {}

        Ok(())
    }
}
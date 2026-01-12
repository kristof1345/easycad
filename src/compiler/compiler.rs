use crate::State;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

pub struct Compiler {
    params: HashMap<String, f64>,
    points: HashMap<String, Point>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            points: HashMap::new(),
        }
    }

    pub fn process_line(
        &mut self,
        _state: &mut State,
        _line: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!("process_line inside of Compiler to process .cad files");
        // Err("not implemented".into())
    }
}

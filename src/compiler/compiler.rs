// used to compile .cad files
use crate::model::circle::CircleOps;
use crate::model::line::LineOps;
use crate::State;
use anyhow::{anyhow, Context, Result};
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

    fn eval_exp(&self, expr_string: &str) -> Result<f64> {
        let expr: meval::Expr = expr_string.parse().context("Invalid math expression")?;
        expr.eval_with_context(&self.params)
            .context("Failed to evaluate expression")
    }

    pub fn process_line(&mut self, state: &mut State, line: &str, line_num: usize) -> Result<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() || parts[0].starts_with("//") {
            return Ok(());
        }

        match parts[0] {
            // param width = 6000
            "param" => {
                if parts.len() < 4 || parts[2] != "=" {
                    return Err(anyhow!(
                        "Error on line {line_num}. Usage: param <name> = <expression>"
                    ));
                }
                let name = parts[1].to_string();
                let math_str = parts[3..].join(" ");
                let value = self.eval_exp(&math_str)?;

                self.params.insert(name, value);
            }
            // point A 0 0
            "point" => {
                if parts.len() < 4 {
                    return Err(anyhow!(
                        "Error on line {line_num}. Usage: point <name> <expression> <expression>"
                    ));
                }
                let name = parts[1].to_string();
                let value_x = self.eval_exp(parts[2])?;
                let value_y = self.eval_exp(parts[3])?;

                self.points.insert(
                    name,
                    Point {
                        x: value_x,
                        y: value_y,
                    },
                );
            }
            // line bottom A B
            "line" => {
                if parts.len() < 4 {
                    return Err(anyhow!(
                        "Error on line {line_num}. Usage: line <name> <name_of_x_point> <name_of_y_point>"
                    ));
                }

                let p1 = self.points.get(parts[2]).ok_or_else(|| {
                    anyhow!("Error on line {line_num}. Unknown point: {}", parts[2])
                })?;
                let p2 = self.points.get(parts[3]).ok_or_else(|| {
                    anyhow!("Error on line {line_num}. Unknown point: {}", parts[3])
                })?;

                state.add_line(
                    [p1.x as f32, p1.y as f32],
                    [p2.x as f32, p2.y as f32],
                    false,
                );
            }
            // circle name point radius
            "circle" => {
                if parts.len() < 4 {
                    return Err(anyhow!(
                        "Error on line {line_num}. Usage: circle <name> <center_point> <expression>"
                    ));
                }

                let center = self.points.get(parts[2]).ok_or_else(|| {
                    anyhow!("Error on line {line_num}. Unknown point: {}", parts[2])
                })?;
                let radius = self.eval_exp(parts[3])?;

                state.add_circle(
                    [center.x as f32, center.y as f32],
                    radius as f32,
                    [1.0, 1.0, 1.0],
                    false,
                    false,
                    false,
                );
            }
            _ => {}
        }

        Ok(())
    }
}

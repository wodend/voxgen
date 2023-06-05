use enterpolation::{linear::ConstEquidistantLinear, Curve};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::value;
use nom::multi::many0;
use nom::sequence::separated_pair;
use nom::IResult;
use palette::{LinSrgba, Srgba};
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;

use crate::turtle::{self, TurtleGraphics};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Command {
    Draw,
    Step,
    Left,
    Right,
    DrawLeft,
    DrawRight,
    SubfigureA,
    SubfigureB,
}

fn parse_sentence(sentence: &str) -> IResult<&str, Vec<Command>> {
    many0(alt((
        value(Command::Draw, tag("F")),
        value(Command::Step, tag("f")),
        value(Command::Left, tag("+")),
        value(Command::Right, tag("-")),
        value(Command::DrawLeft, tag("L")),
        value(Command::DrawRight, tag("R")),
        value(Command::SubfigureA, tag("A")),
        value(Command::SubfigureB, tag("B")),
    )))(sentence)
}

fn parse_productions(rules: Vec<&str>) -> IResult<&str, HashMap<Command, Vec<Command>>> {
    let mut output = HashMap::new();
    for rule in rules {
        let pair = separated_pair(parse_sentence, tag("→"), parse_sentence)(rule)?;
        output.insert(pair.1 .0[0], pair.1 .1);
    }
    Ok(("", output))
}

#[derive(Debug)]
pub struct LSystem {
    name: String,
    axiom: Vec<Command>,
    productions: HashMap<Command, Vec<Command>>,
}

impl LSystem {
    pub fn new(name: &str, axiom: &str, productions: Vec<&str>) -> LSystem {
        LSystem {
            name: name.to_string(),
            axiom: parse_sentence(axiom).unwrap().1,
            productions: parse_productions(productions).unwrap().1,
        }
    }

    fn derive(&self, sentence: &Vec<Command>, n: u32) -> Vec<Command> {
        if n == 0 {
            sentence.clone()
        } else {
            let mut derivation = Vec::new();
            for c in sentence {
                if self.productions.contains_key(c) {
                    derivation.extend(self.productions[c].clone());
                } else {
                    derivation.push(*c);
                }
            }
            self.derive(&derivation, n - 1)
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn commands(&self, n: u32) -> Vec<Command> {
        self.derive(&self.axiom, n)
    }
}

/// Render an L System string in 3D with it's turtle intepretation.
pub struct RenderOptions {
    derivation_length: u32,
    step_size: f32,
    angle_increment: f32,
    size_x: u32,
    size_y: u32,
    size_z: u32,
    offset_x: f32,
    offset_y: f32,
    offset_z: f32,
    rainbow: bool,
}

impl RenderOptions {
    pub fn new() -> RenderOptions {
        RenderOptions {
            derivation_length: 2,
            step_size: 2.0,
            angle_increment: std::f32::consts::FRAC_PI_2,
            size_x: 64,
            size_y: 64,
            size_z: 64,
            offset_x: 0.0,
            offset_y: 0.0,
            offset_z: 0.0,
            rainbow: false,
        }
    }

    pub fn step_size(&mut self, d: f32) -> &mut Self {
        self.step_size = d;
        self
    }

    pub fn derivation_length(&mut self, n: u32) -> &mut Self {
        self.derivation_length = n;
        self
    }

    pub fn angle_increment(&mut self, delta: f32) -> &mut Self {
        self.angle_increment = delta;
        self
    }

    pub fn size_x(&mut self, size_x: u32) -> &mut Self {
        self.size_x = size_x;
        self
    }

    pub fn size_y(&mut self, size_y: u32) -> &mut Self {
        self.size_y = size_y;
        self
    }

    pub fn offset_x(&mut self, offset: f32) -> &mut Self {
        self.offset_x = offset;
        self
    }

    pub fn offset_y(&mut self, offset: f32) -> &mut Self {
        self.offset_y = offset;
        self
    }

    pub fn get_rainbow(&self, len: usize) -> Vec<[u8; 4]> {
        let curve = ConstEquidistantLinear::<f32, _, 7>::equidistant_unchecked([
            LinSrgba::new(1.0, 0.0, 0.0, 1.0),
            LinSrgba::new(1.0, 1.0, 0.0, 1.0),
            LinSrgba::new(0.0, 1.0, 0.0, 1.0),
            LinSrgba::new(0.0, 1.0, 1.0, 1.0),
            LinSrgba::new(0.0, 0.0, 1.0, 1.0),
            LinSrgba::new(1.0, 0.0, 1.0, 1.0),
            LinSrgba::new(1.0, 0.0, 0.0, 1.0),
        ]);
        let mut gradient: Vec<[u8; 4]> = Vec::new();
        for srgba in curve.take(len) {
            let rgba = Srgba::from_linear(srgba).into();
            gradient.push(rgba);
        }
        gradient
    }

    pub fn rainbow(&mut self, rainbow: bool) -> &mut Self {
        self.rainbow = rainbow;
        self
    }

    fn draw(&self, turtle: &mut TurtleGraphics, c: Command) {
        match c {
            Command::Step => turtle.step(self.step_size),
            Command::Draw => turtle.draw(self.step_size),
            Command::Left => turtle.left(self.angle_increment),
            Command::Right => turtle.right(self.angle_increment),
            Command::DrawLeft => {
                turtle.draw(self.step_size);
                turtle.left(self.angle_increment);
                turtle.draw(self.step_size);
            }
            Command::DrawRight => {
                turtle.draw(self.step_size);
                turtle.right(self.angle_increment);
                turtle.draw(self.step_size);
            }
            _ => (),
        }
    }

    fn draw_gradient(&self, turtle: &mut TurtleGraphics, c: Command, colors: &[[u8; 4]]) {
        match c {
            Command::Step => turtle.step(self.step_size),
            Command::Draw => turtle.draw_gradient(self.step_size, colors),
            Command::Left => turtle.left(self.angle_increment),
            Command::Right => turtle.right(self.angle_increment),
            Command::DrawLeft => {
                turtle.draw_gradient(self.step_size, &colors[0..3]);
                turtle.left(self.angle_increment);
                turtle.draw_gradient(self.step_size, &colors[3..6]);
            }
            Command::DrawRight => {
                turtle.draw_gradient(self.step_size, &colors[0..3]);
                turtle.right(self.angle_increment);
                turtle.draw_gradient(self.step_size, &colors[3..6]);
            }
            _ => (),
        }
    }

    fn draw_color(&self, turtle: &mut TurtleGraphics, c: Command, color: &[u8; 4]) {
        match c {
            Command::Step => turtle.step(self.step_size),
            Command::Draw => turtle.draw_color(self.step_size, color),
            Command::Left => turtle.left(self.angle_increment),
            Command::Right => turtle.right(self.angle_increment),
            Command::DrawLeft => {
                turtle.draw_color(self.step_size, color);
                turtle.left(self.angle_increment);
                turtle.draw_color(self.step_size, color);
            }
            Command::DrawRight => {
                turtle.draw_color(self.step_size, color);
                turtle.right(self.angle_increment);
                turtle.draw_color(self.step_size, color);
            }
            _ => (),
        }
    }

    pub fn render(&self, l_system: LSystem) {
        let mut turtle = TurtleGraphics::new(self.size_x, self.size_y, self.size_z);
        // Initialize the turtle in the center of the canvas.
        turtle.step(self.size_x as f32 / 2.0);
        turtle.left(std::f32::consts::FRAC_PI_2);
        turtle.step(self.size_y as f32 / 2.0);
        // Offset per configuration.
        turtle.step(self.offset_y);
        turtle.right(std::f32::consts::FRAC_PI_2);
        turtle.step(self.offset_x);
        turtle.left(std::f32::consts::FRAC_PI_2);

        let commands = l_system.commands(self.derivation_length);
        let mut i = 0;
        let r = self.get_rainbow(250);
        for c in &commands {
            match c {
                Command::Step => (),
                Command::Left => (),
                Command::Right => (),
                _ => {
                    if i < 250 - 1 {
                        i += 1
                    } else {
                    };
                }
            }
            if self.rainbow {
                self.draw_color(&mut turtle, *c, &r[i]);
            } else {
                self.draw(&mut turtle, *c);
            }
        }
        turtle
            .buf()
            .save(format!(
                "test/volumes/{}_{}.vox",
                l_system.name(),
                self.derivation_length
            ))
            .unwrap();
    }
}

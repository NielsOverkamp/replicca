use crate::maneuver::Move;
use json::JsonValue;
use std::convert::TryInto;

#[derive(Copy, Clone, Debug)]
pub struct Coordinate {
    x: i64,
    y: i64,
    z: i64,
}

impl Coordinate {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        return Self {
            x,
            y,
            z,
        };
    }

    pub fn delta_mut(&mut self, x: i64, y: i64, z: i64) {
        self.x += x;
        self.y += y;
        self.z += z;
    }

    pub fn delta(&self, x: i64, y: i64, z: i64) -> Self {
        Coordinate::new(self.x + x, self.y + y, self.z + z)
    }
}

impl Default for Coordinate {
    fn default() -> Self {
        return Self { x: 0, y: 0, z: 0 };
    }
}

impl Into<JsonValue> for Coordinate {
    fn into(self) -> JsonValue {
        json::object! {
            x: Into::<JsonValue>::into(self.x),
            y: Into::<JsonValue>::into(self.y),
            z: Into::<JsonValue>::into(self.z),
        }
    }
}

fn i64_from_json(v: &JsonValue) -> i64 {
    v.as_i64().expect("Expected json number")
}

impl From<&JsonValue> for Coordinate {
    fn from(jv: &JsonValue) -> Self {
        if let JsonValue::Object(o) = jv {
            Self {
                x: i64_from_json(&o["x"]),
                y: i64_from_json(&o["y"]),
                z: i64_from_json(&o["z"]),
            }
        } else {
            panic!("Expected to get json object, got {}", jv)
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn as_index(self) -> u8 {
        match self {
            Direction::North => 0,
            Direction::East => 1,
            Direction::South => 2,
            Direction::West => 3
        }
    }

    pub fn from_index(index: u8) -> Self {
        match index {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            _ => panic!("Expected direction index (0-3), got {}", index)
        }
    }

    // Positive for clockwise, negative for counter clockwise
    pub fn turn(self, count: i8) -> Self {
        Self::from_index((self.as_index() as i8 + count).rem_euclid(4) as u8)
    }
}

impl Into<JsonValue> for Direction {
    fn into(self) -> JsonValue {
        let s = match self {
            Direction::North => "N",
            Direction::East => "E",
            Direction::South => "S",
            Direction::West => "W"
        };
        return s.into();
    }
}

impl From<&JsonValue> for Direction {
    fn from(jv: &JsonValue) -> Self {
        if jv.is_string() {
            match jv.as_str().unwrap() {
                "N" | "n" => Direction::North,
                "E" | "e" => Direction::East,
                "S" | "s" => Direction::South,
                "W" | "w" => Direction::West,
                s => panic!("Expected N, E, S or W, got {}", s)
            }
        } else {
            panic!("Expected json string for direction enum, got {}", jv)
        }
    }
}

#[derive(Debug)]
pub struct Position {
    coordinate: Coordinate,
    direction: Direction,
}

impl Position {
    pub fn turn(&mut self, count: i8) {
        self.direction = self.direction.turn(count);
    }

    pub fn move_horizontal(&mut self, count: i64) {
        match self.direction {
            Direction::North => self.coordinate.delta_mut(0, 0, -count),
            Direction::East => self.coordinate.delta_mut(count, 0, 0),
            Direction::South => self.coordinate.delta_mut(0, 0, count),
            Direction::West => self.coordinate.delta_mut(-count, 0, 0),
        };
    }

    pub fn move_vertical(&mut self, count: i64) {
        self.coordinate.delta_mut(0, count, 0);
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            coordinate: Coordinate::default(),
            direction: Direction::North,
        }
    }
}

impl Into<JsonValue> for &Position {
    fn into(self) -> JsonValue {
        json::object! {
            coordinate: Into::<JsonValue>::into(self.coordinate),
            direction: Into::<JsonValue>::into(self.direction),
        }
    }
}

impl From<&JsonValue> for Position {
    fn from(jv: &JsonValue) -> Self {
        if let JsonValue::Object(o) = jv {
            Self {
                coordinate: (&o["coordinate"]).into(),
                direction: (&o["direction"]).into(),
            }
        } else {
            panic!("Expected json object for position, got {}", jv)
        }
    }
}

#[derive(Debug)]
pub struct Item {
    count: u8,
    name: String,
}

impl Into<JsonValue> for &Item {
    fn into(self) -> JsonValue {
        json::object! {
            count: self.count,
            name: self.name.clone(),
        }
    }
}

impl From<&JsonValue> for Item {
    fn from(jv: &JsonValue) -> Self {
        if let JsonValue::Object(o) = jv {
            Self {
                count: o["count"].as_u8().expect("Expected number"),
                name: o["name"].as_str().expect("Expected string").to_string(),
            }
        } else {
            panic!("Expect json object got {}", jv)
        }
    }
}

#[derive(Debug)]
pub struct Inventory {
    slots: [Option<Item>; 16]
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            slots: [None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, ]
        }
    }

    pub fn coord_to_slot(x: u8, y: u8) -> u8 {
        x + 4 * y
    }
}

impl Into<JsonValue> for &Inventory {
    fn into(self) -> JsonValue {
        let jv_arr = self.slots.iter().map(|s| {
            match s {
                None => JsonValue::Null,
                Some(i) => i.into()
            }
        }).collect();
        return JsonValue::Array(jv_arr);
    }
}

impl From<&JsonValue> for Inventory {
    fn from(jv: &JsonValue) -> Self {
        if let JsonValue::Array(v) = jv {
            let slots: Vec<Option<Item>> = v.iter().map(|s| {
                match s {
                    JsonValue::Null => None,
                    JsonValue::Object(_) => Some(s.into()),
                    _ => panic!("Expected null or object, got {}", s),
                }
            }).collect();
            Self {
                slots: slots.try_into().expect("Expected 16 entries")
            }
        } else {
            panic!("Expected json array, got {}",jv)
        }
    }
}

#[derive(Debug)]
pub struct TurtleState {
    pub label: String,
    pub position: Position,
    pub fuel_level: i64,
    pub inventory: Inventory,
}

impl TurtleState {
    pub fn move_turtle(&mut self, move_type: &Move, count: i64) {
        match move_type {
            Move::Forward => self.position.move_horizontal(count),
            Move::Backward => self.position.move_horizontal(-count),
            Move::Up => self.position.move_vertical(count),
            Move::Down => self.position.move_vertical(-count),
            Move::Left => self.position.turn((-count) as i8),
            Move::Right => self.position.turn(count as i8),
        }
    }
}

impl Default for TurtleState {
    fn default() -> Self {
        Self {
            label: String::new(),
            position: Position::default(),
            fuel_level: 0,
            inventory: Inventory::new()
        }
    }
}

impl Into<JsonValue> for &TurtleState {
    fn into(self) -> JsonValue {
        json::object! {
            label: self.label.clone(),
            position: &self.position,
            fuel_level: self.fuel_level,
            inventory: &self.inventory,
        }
    }
}

impl From<&JsonValue> for TurtleState {
    fn from(jv: &JsonValue) -> Self {
        if let JsonValue::Object(o) = jv {
            Self {
                label: o["label"].as_str().expect("Expected string").to_string(),
                position: (&o["position"]).into(),
                fuel_level: o["fuel_level"].as_i64().expect("Expected number"),
                inventory: (&o["inventory"]).into(),
            }
        } else {
            panic!("Expected json object got {}", jv)
        }
    }
}


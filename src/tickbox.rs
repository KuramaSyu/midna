pub struct TickBox {
    pub lines: Vec<String>,
    pub checked: Vec<u8>,
}

impl TickBox {
    pub fn new<TStr: Into<String>>(lines: Vec<TStr>) -> Self {
        let len = lines.len();
        let lines = lines.into_iter().map(|s| s.into()).collect();
        Self {
            lines,
            checked: vec![0; len],
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            result.push_str(
                match self.checked[i] {
                    0 => "- [ ] ",
                    1 => "- -> ",
                    2 => "- [x] ",
                    _ => panic!("Invalid state"),
                }
            );
            result.push_str(line);
            result.push('\n');
        }
        result
    }

    pub fn toggle(&mut self, field: &str, state: u8) {
        for (i, line) in self.lines.iter().enumerate() {
            if line == field {
                self.checked[i] = state;
                return;
            }
        }
        panic!("{}", format!("Field {field} not found"))
    }

    pub fn next(&mut self) {
        for (i, state) in self.checked.iter().enumerate() {
            if *state == 1 {
                self.checked[i] = 2;
                if i + 1 < self.checked.len() {
                    self.checked[i + 1] = 1;
                }
                return;
            }
        }
    }
}
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::bail;
use regex::Regex;

#[derive(Debug)]
struct Group {
    name: String,
    regex: Regex,
}

struct Groups(Vec<Group>);

impl Groups {
    pub fn group_name(&self, lineitem: &str) -> String {
        String::from("todo")
    }
}

pub fn parse_groups(path: impl AsRef<Path>) -> anyhow::Result<Groups> {
    let mut groups = Groups(Vec::new());
    let file = BufReader::new(File::open(&path)?);
    for line in file.lines() {
        let line = line?;
        let Some((name, regex_str)) = line.split_once(" | ") else {
            bail!(
                "reading {:?}: bad line format (missing |): {line}",
                path.as_ref(),
            );
        };
        groups.0.push(Group {
            name: name.to_string(),
            regex: Regex::new(regex_str)?,
        });
    }
    Ok(groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_groups_fn_correctly_parses_groups_config_file() {
        let groups = parse_groups("testdata/groups").unwrap();
        assert_eq!(
            groups.group_name("The Power of Go: Tests (Go 1.22 edition)"),
            "The Power of Go: Tests"
        );
    }
}

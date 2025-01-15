use anyhow::bail;
use regex::Regex;

use std::{fs::File, io::{BufRead, BufReader}, path::Path};

#[derive(Debug)]
struct Group {
    name: String,
    regex: Regex,
}

#[derive(Default)]
pub struct Groups(Vec<Group>);

impl Groups {
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut groups = Self(Vec::new());
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
    pub fn product_group(&self, lineitem: &str) -> Option<String> {
        self.0
            .iter()
            .find(|g| g.regex.is_match(lineitem))
            .map(|g| g.name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_groups_fn_correctly_parses_groups_config_file() {
        let groups = Groups::from_file("testdata/groups").unwrap();
        assert_eq!(
            groups
                .product_group("The Power of Go: Tests (Go 1.22 edition)")
                .unwrap(),
            "The Power of Go: Tests"
        );
        assert_eq!(
            groups
                .product_group("For the Love of Go (Go 1.23 edition)")
                .unwrap(),
            "For the Love of Go"
        );
        assert_eq!(groups.product_group("bogus product"), None);
        assert_eq!(groups.product_group("For the Love of Go"), None);
    }
}

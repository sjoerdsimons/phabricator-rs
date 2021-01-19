#![allow(dead_code)]
//use anyhow::Result;
use anyhow::{anyhow, bail, ensure, Result};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
enum Param {
    Key(HashMap<String, Param>),
    Values(Vec<String>),
    Value(String),
}

#[derive(Debug)]
pub struct Params(HashMap<String, Param>);

impl Params {
    fn insert_key_value(&mut self, key: &str, value: &str) -> Result<()> {
        let mut m = &mut self.0;

        let mut parts = key
            .split('[')
            .enumerate()
            .map(|(i, v)| {
                if i > 0 {
                    v.strip_suffix(']')
                        .ok_or_else(|| anyhow!("index didn't end in ]"))
                } else {
                    Ok(v)
                }
            })
            .peekable();
        while let Some(p) = parts.next().transpose()? {
            if let Some(next) = parts.peek() {
                let next = match next {
                    Ok(ref n) => n,
                    Err(_) => parts.next().unwrap()?,
                };
                if let Ok(index) = next.parse::<usize>() {
                    match m
                        .entry(p.to_string())
                        .or_insert_with(|| Param::Values(Vec::new()))
                    {
                        Param::Values(ref mut values) => {
                            ensure!(parts.count() == 1, "index parameter not last");
                            ensure!(values.len() == index, "parameter index not in order");
                            values.push(value.to_string());
                            return Ok(());
                        }
                        _ => bail!("Inconsistent parameter hierarchy"),
                    }
                } else {
                    m = match m
                        .entry(p.to_string())
                        .or_insert_with(|| Param::Key(HashMap::new()))
                    {
                        Param::Key(ref mut m) => m,
                        _ => bail!("Invalid parameters"),
                    };
                }
            } else if m
                .insert(p.to_string(), Param::Value(value.to_string()))
                .is_some()
            {
                bail!("Duplicate key in parameters");
            }
        }
        Ok(())
    }

    pub fn new(input: &[u8]) -> Result<Self> {
        let parser = form_urlencoded::parse(input);
        let mut this = Params(HashMap::new());

        for (key, value) in parser {
            this.insert_key_value(&key, &value)?;
        }
        Ok(this)
    }

    fn do_get(&self, key: &[&str]) -> Option<&Param> {
        let mut p: &Param = self.0.get(key[0])?;

        for k in key.iter().skip(1) {
            p = match p {
                Param::Key(v) => v.get(*k)?,
                _ => return None,
            };
        }

        Some(&p)
    }

    pub fn get(&self, key: &[&str]) -> Option<&str> {
        match self.do_get(key) {
            Some(Param::Value(ref s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_values(&self, key: &[&str]) -> Option<&Vec<String>> {
        match self.do_get(key) {
            Some(Param::Values(ref v)) => Some(v),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_value() {
        let p = Params::new(b"key=value").expect("Failed to create params");
        assert_eq!(Some("value"), p.get(&["key"]));
    }

    #[test]
    fn one_level() {
        let p = Params::new(b"key[level]=value").expect("Failed to create params");
        assert_eq!(Some("value"), p.get(&["key", "level"]));
    }

    #[test]
    fn array() {
        let p = Params::new(b"key[0]=value0&key[1]=value1&key[2]=value2")
            .expect("Failed to create params");
        let values = p.get_values(&["key"]).expect("Missing key");
        assert_eq!(["value0", "value1", "value2"], values.as_slice());
    }

    #[test]
    fn one_level_array() {
        let p = Params::new(b"key[bla][0]=value0&key[bla][1]=value1&key[bla][2]=value2")
            .expect("Failed to create params");
        let values = p.get_values(&["key", "bla"]).expect("Missing key");
        assert_eq!(["value0", "value1", "value2"], values.as_slice());
    }

    #[test]
    fn duplicate_value() {
        let p = Params::new(b"key=value0&key=value1");
        assert!(p.is_err());
    }

    #[test]
    fn inconsistent_hierachy() {
        let p = Params::new(b"key[bla]=value0&key[bla][0]=value1");
        assert!(p.is_err());
    }

    #[test]
    fn incorrect_index() {
        let p = Params::new(b"key[bla=value0");
        assert!(p.is_err());
    }
}

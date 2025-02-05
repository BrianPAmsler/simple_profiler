use std::{collections::HashMap, sync::Mutex};

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

static STATIC_STR_MAP: Mutex<Option<HashMap<&str, &'static str>>> = Mutex::new(None);
pub struct StaticStrVisitor ();

impl StaticStrVisitor {
    pub fn new() -> StaticStrVisitor {
        let mut guard = STATIC_STR_MAP.lock().unwrap();

        if (*guard).is_none() {
            *guard = Some(HashMap::new());
        }

        StaticStrVisitor {}
    }
}

impl Visitor<'_> for StaticStrVisitor {
    type Value = StaticStr;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        let mut guard = STATIC_STR_MAP.lock().unwrap();
        let map = (*guard).as_mut().unwrap();

        match map.get(&v) {
            Some(s) => Ok(StaticStr(*s)),
            None => {
                let new_string = v.to_owned().leak();

                map.insert(new_string, new_string);

                Ok(StaticStr(new_string))
            },
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        self.visit_str(&v)
    }
}


#[derive(Serialize, Clone, Copy)]
pub struct StaticStr(pub(in crate) &'static str);

impl<'de> Deserialize<'de> for StaticStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {
        
        deserializer.deserialize_str(StaticStrVisitor::new())
    }
}
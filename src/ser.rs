use serde::de::{Error as DeError, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::Environment;

impl Serialize for Environment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> Result<Environment, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let lowered = raw.to_ascii_lowercase();

        match lowered.as_str() {
            "test" => Ok(Environment::test()),
            "prod" => Ok(Environment::prod()),
            other => Err(D::Error::invalid_value(
                Unexpected::Str(other),
                &r#""test" or "prod" (any casing)"#,
            )),
        }
    }
}

use serde::de::Deserializer;
use serde::Deserialize;

pub mod info;
pub mod search;

fn str_or_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IorS<'a> {
        I(u32),
        S(&'a str),
    }
    let iors = IorS::deserialize(deserializer)?;
    let v = match iors {
        IorS::I(v) => v,
        IorS::S(s) => s.parse().map_err(serde::de::Error::custom)?,
    };

    Ok(v)
}

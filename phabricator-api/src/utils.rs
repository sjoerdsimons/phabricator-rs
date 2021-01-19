use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use serde::de::Deserializer;
use serde::Deserialize;

pub fn str_or_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
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

pub fn deserialize_timestamp<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = i64::deserialize(d)?;
    Ok(Utc.timestamp(s, 0))
}

pub fn deserialize_timestamp_option<'de, D>(d: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<i64>::deserialize(d)?;
    Ok(s.map(|s| Utc.timestamp(s, 0)))
}

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn de_hex_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s: String = Deserialize::deserialize(deserializer)?;

    if s.starts_with("0x") {
        s = s[2..].to_string();
    }

    u32::from_str_radix(s.as_ref(), 16).map_err(serde::de::Error::custom)
}

pub fn de_hex_to_vu8<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s: String = Deserialize::deserialize(deserializer)?;

    if s.starts_with("0x") {
        s = s[2..].to_string();
    }

    if s.len() % 2 != 0 {
        return Err(serde::de::Error::custom("hex string has odd length"));
    }

    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(serde::de::Error::custom))
        .collect()
}

pub fn de_hex_to_ovu8<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s: String = Deserialize::deserialize(deserializer)?;

    if s.starts_with("0x") {
        s = s[2..].to_string();
    }

    if s.len() % 2 != 0 {
        return Err(serde::de::Error::custom("hex string has odd length"));
    }

    let vec: Vec<u8> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(serde::de::Error::custom))
        .collect::<Result<_, D::Error>>()?;

    Ok(Some(vec))
}

pub fn se_u32_to_hex<S>(val: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_string = format!("0x{:08X}", val);
    serializer.serialize_str(&hex_string)
}

pub fn se_vu8_to_hex<S>(val: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut hex_string = String::from("0x");

    for thing in val {
        hex_string.push_str(format!("{:02x}", thing).as_str())
    }

    serializer.serialize_str(&hex_string)
}

pub fn se_ovu8_to_hex<S>(val: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match val {
        Some(v) => se_vu8_to_hex(&v, serializer),
        None => {
            let none: Option<Vec<u8>> = None;
            none.serialize(serializer)
        }
    }
}

pub fn get_none<T>() -> Option<T> {
    None
}

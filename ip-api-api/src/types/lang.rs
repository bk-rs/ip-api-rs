#![allow(non_camel_case_types)]
//! https://members.ip-api.com/docs/json

use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

#[derive(Deserialize_enum_str, Serialize_enum_str, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    EN,
    DE,
    ES,
    #[serde(rename = "pt-BR")]
    PT_BR,
    FR,
    JA,
    #[serde(rename = "zh-CN")]
    ZH_CN,
    RU,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        assert_eq!(Lang::EN.to_string(), "en");
        assert_eq!(Lang::DE.to_string(), "de");
        assert_eq!(Lang::ES.to_string(), "es");
        assert_eq!(Lang::PT_BR.to_string(), "pt-BR");
        assert_eq!(Lang::FR.to_string(), "fr");
        assert_eq!(Lang::JA.to_string(), "ja");
        assert_eq!(Lang::ZH_CN.to_string(), "zh-CN");
        assert_eq!(Lang::RU.to_string(), "ru");
    }
}

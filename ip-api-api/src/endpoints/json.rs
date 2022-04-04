//! https://members.ip-api.com/docs/json
//! https://ip-api.com/docs/api:json

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use http_api_client_endpoint::{
    http::{header::ACCEPT, Method},
    Body, Endpoint, Request, Response, MIME_APPLICATION_JSON,
};
use serde::{de, Deserialize, Deserializer};
use serde_json::{Map, Value};
use url::Url;

use crate::{
    endpoints::{common::EndpointError, helper::get_n_from_headers_by_key, URL_BASE, URL_BASE_PRO},
    objects::rate_limit::{RateLimit, RESPONSE_HEADER_KEY_X_RL, RESPONSE_HEADER_KEY_X_TTL},
    types::lang::Lang,
};

//
#[derive(Debug, Clone)]
pub struct Json {
    pub query: Box<str>,
    pub key: Option<Box<str>>,
    pub fields: Option<Box<str>>,
    pub lang: Option<Lang>,
}

impl Json {
    pub fn new(query: impl AsRef<str>, key: Option<Box<str>>) -> Self {
        Self {
            query: query.as_ref().into(),
            key,
            fields: None,
            lang: None,
        }
    }

    pub fn fields(mut self, fields: impl AsRef<str>) -> Self {
        self.fields = Some(fields.as_ref().into());
        self
    }

    pub fn lang(mut self, lang: Lang) -> Self {
        self.lang = Some(lang);
        self
    }
}

impl Endpoint for Json {
    type RenderRequestError = EndpointError;

    type ParseResponseOutput = (JsonResponseBodyJson, Option<RateLimit>);
    type ParseResponseError = EndpointError;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError> {
        let url = format!(
            "{}/json/{}",
            if self.key.is_some() {
                URL_BASE_PRO
            } else {
                URL_BASE
            },
            self.query
        );
        let mut url = Url::parse(url.as_str()).map_err(EndpointError::MakeRequestUrlFailed)?;

        if let Some(key) = &self.key {
            url.query_pairs_mut().append_pair("key", key);
        }
        if let Some(fields) = &self.fields {
            url.query_pairs_mut().append_pair("fields", fields);
        }
        if let Some(lang) = &self.lang {
            url.query_pairs_mut()
                .append_pair("lang", lang.to_string().as_str());
        }

        let request = Request::builder()
            .method(Method::GET)
            .uri(url.as_str())
            .header(ACCEPT, MIME_APPLICATION_JSON)
            .body(vec![])
            .map_err(EndpointError::MakeRequestFailed)?;

        Ok(request)
    }

    fn parse_response(
        &self,
        response: Response<Body>,
    ) -> Result<Self::ParseResponseOutput, Self::ParseResponseError> {
        let json = serde_json::from_slice(response.body())
            .map_err(EndpointError::DeResponseBodyJsonFailed)?;

        let rate_limit = if self.key.is_some() {
            None
        } else {
            Some(RateLimit {
                remaining: get_n_from_headers_by_key(response.headers(), RESPONSE_HEADER_KEY_X_RL)
                    .ok(),
                seconds_until_reset: get_n_from_headers_by_key(
                    response.headers(),
                    RESPONSE_HEADER_KEY_X_TTL,
                )
                .ok(),
            })
        };

        Ok((json, rate_limit))
    }
}

//
//
//
#[derive(Debug, Clone)]
pub enum JsonResponseBodyJson {
    Success(Box<JsonResponseBodySuccessJson>),
    Fail(JsonResponseBodyFailJson),
}

impl<'de> Deserialize<'de> for JsonResponseBodyJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = Map::deserialize(deserializer)?;

        if let Some(status) = map.get("status") {
            let status: Box<str> = Deserialize::deserialize(status).map_err(de::Error::custom)?;
            match status.as_ref() {
                "success" => JsonResponseBodySuccessJson::deserialize(Value::Object(map))
                    .map(|x| JsonResponseBodyJson::Success(x.into()))
                    .map_err(de::Error::custom),
                "fail" => JsonResponseBodyFailJson::deserialize(Value::Object(map))
                    .map(JsonResponseBodyJson::Fail)
                    .map_err(de::Error::custom),
                s => Err(de::Error::custom(format!("status [{}] mismatch", s))),
            }
        } else if map.get("message").is_some() {
            JsonResponseBodyFailJson::deserialize(Value::Object(map))
                .map(JsonResponseBodyJson::Fail)
                .map_err(de::Error::custom)
        } else {
            JsonResponseBodySuccessJson::deserialize(Value::Object(map))
                .map(|x| JsonResponseBodyJson::Success(x.into()))
                .map_err(de::Error::custom)
        }
    }
}

impl JsonResponseBodyJson {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    pub fn as_success(&self) -> Option<&JsonResponseBodySuccessJson> {
        match self {
            Self::Success(x) => Some(x),
            Self::Fail(_) => None,
        }
    }

    pub fn as_fail(&self) -> Option<&JsonResponseBodyFailJson> {
        match self {
            Self::Success(_) => None,
            Self::Fail(x) => Some(x),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonResponseBodySuccessJson {
    #[serde(default = "serde_field_default::default_ip_addr")]
    pub query: IpAddr,
    //
    #[serde(default)]
    pub continent: Box<str>,
    #[serde(default, rename = "continentCode")]
    pub continent_code: Box<str>,
    //
    #[serde(default)]
    pub country: Box<str>,
    #[serde(default, rename = "countryCode")]
    pub country_code: Box<str>,
    #[serde(default, rename = "countryCode3")]
    pub country_code3: Box<str>,
    //
    #[serde(default)]
    pub region: Box<str>,
    #[serde(default, rename = "regionName")]
    pub region_name: Box<str>,
    //
    #[serde(default)]
    pub city: Box<str>,
    #[serde(default)]
    pub district: Box<str>,
    //
    #[serde(default)]
    pub zip: Box<str>,
    //
    #[serde(default)]
    pub lat: f64,
    #[serde(default)]
    pub lon: f64,
    //
    #[serde(
        default = "serde_field_default::chrono_tz::default_tz",
        deserialize_with = "serde_field_with::from_str"
    )]
    pub timezone: Tz,
    #[serde(default)]
    pub offset: isize,
    #[serde(
        default = "serde_field_default::chrono::default_date_time_utc",
        rename = "currentTime"
    )]
    pub current_time: DateTime<Utc>,
    //
    #[serde(default)]
    pub currency: Box<str>,
    //
    #[serde(default, rename = "callingCode")]
    pub calling_code: Box<str>,
    //
    #[serde(default)]
    pub isp: Box<str>,
    #[serde(default)]
    pub org: Box<str>,
    #[serde(default)]
    pub r#as: Box<str>,
    #[serde(default)]
    pub asname: Box<str>,
    #[serde(default)]
    pub reverse: Box<str>,
    //
    #[serde(default)]
    pub mobile: bool,
    #[serde(default)]
    pub proxy: bool,
    #[serde(default)]
    pub hosting: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonResponseBodyFailJson {
    #[serde(default)]
    pub query: Box<str>,
    //
    pub message: Box<str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_request() {
        let json = Json::new("24.48.0.1", None);
        let req = json.render_request().unwrap();
        assert_eq!(req.uri(), "http://ip-api.com/json/24.48.0.1");

        //
        let json = Json::new("24.48.0.1", Some("foo".into()));
        let req = json.render_request().unwrap();
        assert_eq!(req.uri(), "https://pro.ip-api.com/json/24.48.0.1?key=foo");

        let json = json.fields("status,message,country,query");
        let req = json.render_request().unwrap();
        assert_eq!(
            req.uri(),
            "https://pro.ip-api.com/json/24.48.0.1?key=foo&fields=status%2Cmessage%2Ccountry%2Cquery"
        );

        let json = json.lang(Lang::EN);
        let req = json.render_request().unwrap();
        assert_eq!(
            req.uri(),
            "https://pro.ip-api.com/json/24.48.0.1?key=foo&fields=status%2Cmessage%2Ccountry%2Cquery&lang=en"
        );
    }

    #[test]
    fn test_de_response_body_json() {
        match serde_json::from_str::<JsonResponseBodyJson>(include_str!(
            "../../tests/response_body_json_files/json_default.json"
        )) {
            Ok(JsonResponseBodyJson::Success(ok_json)) => {
                assert_eq!(ok_json.query.to_string(), "24.48.0.1");
                assert_eq!(ok_json.continent_code, "".into());
                assert_eq!(ok_json.country_code, "CA".into());
            }
            ret => panic!("{:?}", ret),
        }

        match serde_json::from_str::<JsonResponseBodyJson>(include_str!(
            "../../tests/response_body_json_files/json_full_fields.json"
        )) {
            Ok(JsonResponseBodyJson::Success(ok_json)) => {
                assert_eq!(ok_json.query.to_string(), "24.48.0.1");
                assert_eq!(ok_json.continent_code, "NA".into());
                assert_eq!(ok_json.country_code, "CA".into());
            }
            ret => panic!("{:?}", ret),
        }

        match serde_json::from_str::<JsonResponseBodyJson>(include_str!(
            "../../tests/response_body_json_files/json_full_fields_and_zh-CN_lang.json"
        )) {
            Ok(JsonResponseBodyJson::Success(ok_json)) => {
                assert_eq!(ok_json.query.to_string(), "24.48.0.1");
                assert_eq!(ok_json.continent_code, "NA".into());
                assert_eq!(ok_json.country_code, "CA".into());
            }
            ret => panic!("{:?}", ret),
        }

        //
        match serde_json::from_str::<JsonResponseBodyJson>(include_str!(
            "../../tests/response_body_json_files/json_err_1.json"
        )) {
            Ok(JsonResponseBodyJson::Fail(err_json)) => {
                assert_eq!(err_json.query, "".into());
            }
            ret => panic!("{:?}", ret),
        }

        match serde_json::from_str::<JsonResponseBodyJson>(include_str!(
            "../../tests/response_body_json_files/json_err_2.json"
        )) {
            Ok(JsonResponseBodyJson::Fail(err_json)) => {
                assert_eq!(err_json.query, "24".into());
            }
            ret => panic!("{:?}", ret),
        }

        match serde_json::from_str::<JsonResponseBodyJson>(include_str!(
            "../../tests/response_body_json_files/json_err_3.json"
        )) {
            Ok(JsonResponseBodyJson::Fail(err_json)) => {
                assert_eq!(err_json.query, "".into());
            }
            ret => panic!("{:?}", ret),
        }
    }
}

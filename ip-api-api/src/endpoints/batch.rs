//! https://members.ip-api.com/docs/batch

use core::ops::Deref;

use http_api_client_endpoint::{
    http::{
        header::{ACCEPT, CONTENT_TYPE},
        Method,
    },
    Body, Endpoint, Request, Response, MIME_APPLICATION_JSON,
};
use serde::Deserialize;
use serde_json::{Map, Value};
use url::Url;

use crate::{
    endpoints::{
        common::EndpointError, helper::get_n_from_headers_by_key, json::JsonResponseBodyJson,
        URL_BASE, URL_BASE_PRO,
    },
    objects::rate_limit::{RateLimit, RESPONSE_HEADER_KEY_X_RL, RESPONSE_HEADER_KEY_X_TTL},
    types::lang::Lang,
};

pub const MAX_QUERY: usize = 100;

//
#[derive(Debug, Clone)]
pub struct Batch {
    pub queries: Vec<BatchQuery>,
    pub key: Option<Box<str>>,
    pub fields: Option<Box<str>>,
    pub lang: Option<Lang>,
}

#[derive(Debug, Clone)]
pub struct BatchQuery {
    pub query: Box<str>,
    pub fields: Option<Box<str>>,
    pub lang: Option<Lang>,
}
impl BatchQuery {
    pub fn new(query: impl AsRef<str>) -> Self {
        Self {
            query: query.as_ref().into(),
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

impl Batch {
    pub fn new(queries: Vec<BatchQuery>, key: Option<Box<str>>) -> Self {
        if queries.len() > MAX_QUERY {
            debug_assert!(false, "containing up to 100 IP addresses or objects");
        }

        Self {
            queries,
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

impl Endpoint for Batch {
    type RenderRequestError = EndpointError;

    type ParseResponseOutput = (BatchResponseBodyJson, Option<RateLimit>);
    type ParseResponseError = EndpointError;

    fn render_request(&self) -> Result<Request<Body>, Self::RenderRequestError> {
        let url = format!(
            "{}/batch",
            if self.key.is_some() {
                URL_BASE_PRO
            } else {
                URL_BASE
            },
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

        let body_array = self
            .queries
            .iter()
            .map(|x| {
                if x.fields.is_none() && x.lang.is_none() {
                    Value::String(x.query.to_string())
                } else {
                    let mut map = Map::new();
                    map.insert("query".to_owned(), Value::String(x.query.to_string()));
                    if let Some(fields) = &x.fields {
                        map.insert("fields".to_owned(), Value::String(fields.to_string()));
                    }
                    if let Some(lang) = &x.lang {
                        map.insert("lang".to_owned(), Value::String(lang.to_string()));
                    }
                    Value::Object(map)
                }
            })
            .collect::<Vec<_>>();

        let body =
            serde_json::to_vec(&body_array).map_err(EndpointError::SerRequestBodyJsonFailed)?;

        let request = Request::builder()
            .method(Method::POST)
            .uri(url.as_str())
            .header(CONTENT_TYPE, MIME_APPLICATION_JSON)
            .header(ACCEPT, MIME_APPLICATION_JSON)
            .body(body)
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
#[derive(Deserialize, Debug, Clone)]
pub struct BatchResponseBodyJson(pub Vec<JsonResponseBodyJson>);

impl Deref for BatchResponseBodyJson {
    type Target = Vec<JsonResponseBodyJson>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_render_request() {
        let batch = Batch::new(vec![BatchQuery::new("24.48.0.1")], None);
        let req = batch.render_request().unwrap();
        assert_eq!(req.uri(), "http://ip-api.com/batch");
        assert_eq!(req.body(), br#"["24.48.0.1"]"#);

        //
        let batch = Batch::new(
            vec![
                BatchQuery::new("24.48.0.1"),
                BatchQuery::new("8.8.8.8")
                    .fields("country,query")
                    .lang(Lang::EN),
            ],
            Some("foo".into()),
        );
        let req = batch.render_request().unwrap();
        assert_eq!(req.uri(), "https://pro.ip-api.com/batch?key=foo");
        assert_eq!(
            req.body(),
            json! {
                [
                    "24.48.0.1",
                    {"query":"8.8.8.8", "fields":"country,query", "lang":"en"}
                ]
            }
            .to_string()
            .as_bytes()
        );

        let batch = batch.fields("status,message,country,query");
        let req = batch.render_request().unwrap();
        assert_eq!(
            req.uri(),
            "https://pro.ip-api.com/batch?key=foo&fields=status%2Cmessage%2Ccountry%2Cquery"
        );

        let batch = batch.lang(Lang::EN);
        let req = batch.render_request().unwrap();
        assert_eq!(
            req.uri(),
            "https://pro.ip-api.com/batch?key=foo&fields=status%2Cmessage%2Ccountry%2Cquery&lang=en"
        );
    }

    #[test]
    fn test_de_response_body_json() {
        match serde_json::from_str::<BatchResponseBodyJson>(include_str!(
            "../../tests/response_body_json_files/batch_simple.json"
        )) {
            Ok(json) => {
                assert_eq!(json.len(), 3);
                match &json[0] {
                    JsonResponseBodyJson::Success(ok_json) => {
                        assert_eq!(ok_json.query.to_string(), "208.80.152.201")
                    }
                    x => panic!("{:?}", x),
                }
                match &json[1] {
                    JsonResponseBodyJson::Success(ok_json) => {
                        assert_eq!(ok_json.query.to_string(), "8.8.8.8")
                    }
                    x => panic!("{:?}", x),
                }
                match &json[2] {
                    JsonResponseBodyJson::Success(ok_json) => {
                        assert_eq!(ok_json.query.to_string(), "24.48.0.1")
                    }
                    x => panic!("{:?}", x),
                }
            }
            ret => panic!("{:?}", ret),
        }

        match serde_json::from_str::<BatchResponseBodyJson>(include_str!(
            "../../tests/response_body_json_files/batch_simple_with_part_err.json"
        )) {
            Ok(json) => {
                assert_eq!(json.len(), 2);
                match &json[0] {
                    JsonResponseBodyJson::Success(ok_json) => {
                        assert_eq!(ok_json.query.to_string(), "208.80.152.201")
                    }
                    x => panic!("{:?}", x),
                }
                match &json[1] {
                    JsonResponseBodyJson::Fail(err_json) => {
                        assert_eq!(err_json.query, "2".into())
                    }
                    x => panic!("{:?}", x),
                }
            }
            ret => panic!("{:?}", ret),
        }
    }
}

use std::{collections::HashMap, time::Instant};

use base64::prelude::*;
use rand::RngCore;
use reqwest::Client as HttpClient;
use serde_json::{Value, json};

use crate::{Error, LoginMode, encryption::Encryption};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/153.0.0.0 Safari/537.36";
// const USER_AGENT_MOBILE: &str = "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/155.0.0.0 Mobile Safari/537.36";

#[derive(Debug, Clone)]
pub struct Communication {
    http: HttpClient,
    request_number: u32,

    pub root_url: String,
    pub session_id: u64,
    pub space_id: u8,
    pub encryption: Encryption,
    pub last_ping: Instant,
}

impl Communication {
    pub fn new(root_url: &str) -> Result<Self, Error> {
        // root_url = https://example.com/pronote

        let http = HttpClient::builder()
            .cookie_store(true) // CASTGC cookie
            .user_agent(USER_AGENT)
            .build()
            .map_err(|_| Error::ConnectionFailed)?;

        Ok(Self {
            http,
            request_number: 1,
            root_url: root_url.to_owned(),
            session_id: 0,
            space_id: 0,
            encryption: Encryption::new(),
            last_ping: Instant::now(),
        })
    }

    pub fn mock() -> Self {
        Self {
            root_url: String::new(),
            http: HttpClient::new(),
            session_id: 0,
            space_id: 0,
            encryption: Encryption::new(),
            request_number: 1,
            last_ping: Instant::now(),
        }
    }

    pub async fn initialise(
        &mut self,
        login_mode: LoginMode,
        nav_identifier: Option<&str>,
    ) -> Result<Value, Error> {
        let url = match login_mode {
            LoginMode::Password => format!("{}/eleve.html?login=true", self.root_url),
            LoginMode::Token => format!("{}/mobile.eleve.html?login=true", self.root_url),
            LoginMode::QrCode => format!(
                "{}/mobile.eleve.html?fd=1&bydlg=A6ABB224-12DD-4E31-AD3E-8A39A1C2C335&login=true",
                self.root_url
            ), // idk why but it's like that in pronotepy so...
        };
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|_| Error::ConnectionFailed)?;
        let html = response.text().await.map_err(|_| Error::ConnectionFailed)?;
        let attributes = parse_start(&html)?;

        // there is a javascript with a json like this
        self.session_id = attributes
            .get("h")
            .ok_or(Error::InvalidStartPage)?
            .parse()
            .map_err(|_| Error::InvalidStartPage)?;
        self.space_id = attributes
            .get("a")
            .ok_or(Error::InvalidStartPage)?
            .parse()
            .map_err(|_| Error::InvalidStartPage)?;

        // space 3 = students
        // space 6 = students (mobile) ig
        if self.space_id != 3 && self.space_id != 6 {
            return Err(Error::UnsupportedClient);
        }

        // random iv -> send its base64 to server
        let mut new_iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut new_iv);

        let uuid = BASE64_STANDARD.encode(new_iv);
        let res = self
            .post(
                "FonctionParametres",
                json!({
                    "data": {
                        "identifiantNav": nav_identifier,
                        "Uuid": uuid,
                    }
                }),
            )
            .await;

        self.encryption.set_iv(&new_iv);

        res
    }

    pub async fn post(&mut self, function: &str, data: Value) -> Result<Value, Error> {
        let encrypted_no = self.encrypted_request_number()?;
        let body = json!({
            "session": self.session_id,
            "no": encrypted_no,
            "id": function,
            "dataSec": data,
        });

        let url = format!(
            "{}/appelfonction/{}/{}/{}",
            self.root_url, self.space_id, self.session_id, encrypted_no,
        );
        // should've sent encrypted no in url did only send plaintext number
        // apparently pronote's servers don't bother checking that...

        let response = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|_| Error::ConnectionFailed)?
            .json::<Value>()
            .await
            .map_err(|_| Error::ConnectionFailed)?;

        self.request_number += 2; // the server one counts too!
        self.last_ping = Instant::now();

        self.check_error(&response)?;

        Ok(response)
    }

    fn encrypted_request_number(&self) -> Result<String, Error> {
        let number = self.request_number.to_string();
        let encrypted = self.encryption.encrypt(number.as_bytes());

        Ok(hex::encode(encrypted))
    }

    fn check_error(&self, response: &Value) -> Result<(), Error> {
        if let Some(error) = response.get("Erreur") {
            let code = error.get("G").and_then(Value::as_i64).unwrap();
            let message = error
                .get("Titre")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");

            return match code {
                8 | 10 | 13 | 22 => Err(Error::Expired),
                _ => Err(Error::Pronote {
                    code,
                    details: message.to_owned(),
                }),
            };
        }

        if let Some(true) = response
            .pointer("/dataSec/Signature/Erreur")
            .and_then(Value::as_bool)
        {
            let message = response
                .pointer("/dataSec/Signature/MessageErreur")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");

            return Err(Error::Pronote {
                code: -1,
                details: message.to_owned(),
            });
        }

        Ok(())
    }
}

fn parse_start(html: &str) -> Result<HashMap<String, String>, Error> {
    let start = html.find("Start").ok_or(Error::InvalidStartPage)?;
    let html = &html[start..];
    let open = html.find('{').ok_or(Error::InvalidStartPage)?;
    let close = html.find('}').ok_or(Error::InvalidStartPage)?;
    let object = &html[open + 1..close];

    let mut result = HashMap::new();

    for item in object.split(',') {
        let Some((key, value)) = item.split_once(':') else {
            continue;
        };

        result.insert(
            key.trim().trim_matches(['"', '\'']).to_owned(),
            value.trim().trim_matches(['"', '\'']).to_owned(),
        );
    }

    Ok(result)
}

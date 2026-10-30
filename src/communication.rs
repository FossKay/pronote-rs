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
        let (session, space) = parse_session(&html)?;

        // space 3 = students
        // space 6 = students (mobile)
        if space != 3 && space != 6 {
            return Err(Error::UnsupportedClient);
        }

        // there is a javascript with a json like this
        self.session_id = session;
        self.space_id = space;

        // create a random iv, then send its base64 to server
        let mut new_iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut new_iv);

        // i don't know why it's Uuid, it's just NOT that
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

        // AFTER that the request is sent, change the IV used
        // to encrypt request numbers to the new one
        self.encryption.set_iv(&new_iv);

        res
    }

    pub async fn post(&mut self, function: &str, data: Value) -> Result<Value, Error> {
        // every request should have the encrypted request number that
        // prevents replay attacks and authenticates every request,
        // otherwise you could steal the "session" of someone
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

        // count the website request and the server response
        self.request_number += 2;
        self.check_error(&response)?;

        Ok(response)
    }

    fn encrypted_request_number(&self) -> Result<String, Error> {
        let number = self.request_number.to_string();
        let encrypted = self.encryption.encrypt(number.as_bytes());

        // website sends lowercase hex
        // server returns uppercase hex
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
                // TODO: stop guessing
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

fn parse_session(s: &str) -> Result<(u64, u8), Error> {
    // in html: ...Start ({"h":<session>,"a":<space>})...
    let no_demo = s.replace(r#","d":true"#, ""); // only in demo server
    let (h, a) = no_demo
        .split_once(r#"Start ({"h":"#)
        .and_then(|(_, s)| s.split_once(r#","a":"#))
        .ok_or(Error::InvalidStartPage)?;

    Ok((
        h.parse().map_err(|_| Error::InvalidStartPage)?,
        a.split_once("})")
            .ok_or(Error::InvalidStartPage)?
            .0
            .parse()
            .map_err(|_| Error::InvalidStartPage)?,
    ))
}

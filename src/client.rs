use core::str;
use serde_json::{Value, from_value, json};
use std::str::FromStr;

use crate::data::*;
use crate::helpers::*;
use crate::{Error, communication::Communication, encryption::Encryption};

#[derive(Debug, Clone)]
pub struct Client {
    pub username: String,
    pub password: String,
    pub communication: Communication,
    pub parameters: Params,                  // pronote instance parameters
    pub user_parameters: Option<UserParams>, // user parameters, late initialized
    pub nav_identifier: String,              // save for token login
    pub next_token: String,                  // save for token login
    pub uuid: String,                        // save for token login
    pub login_mode: LoginMode,
}

impl Client {
    pub async fn password_login(
        root_url: &str,
        username: &str,
        password: &str,
    ) -> Result<Client, Error> {
        Self::new(root_url, username, password, LoginMode::Password, "", None).await
    }

    pub async fn token_login(
        root_url: &str,
        username: &str,
        token: &str,
        uuid: &str,
        nav_identifier: &str,
    ) -> Result<Client, Error> {
        Self::new(
            root_url,
            username,
            token,
            LoginMode::Token,
            uuid,
            Some(nav_identifier),
        )
        .await
    }

    pub async fn qr_login(qr_code: &str, pin: &str, uuid: &str) -> Result<Client, Error> {
        let qr: QrCode = from_value(serde_json::from_str(qr_code).unwrap()).unwrap();
        let mut encryption = Encryption::new();

        encryption.set_key(pin.as_bytes());
        // actually VERY WEAK security
        // i mean one could easily bruteforce 10000 pins and check which
        // one produce valid lowercase alphanumeric & uppercase hex utf8

        let encrypted_login = hex::decode(&qr.username).unwrap();
        let encrypted_token = hex::decode(&qr.token).unwrap();

        let username_bytes = encryption.try_decrypt(&encrypted_login)?;
        let token_bytes = encryption.try_decrypt(&encrypted_token)?;

        let username = str::from_utf8(&username_bytes).unwrap();
        let token = str::from_utf8(&token_bytes).unwrap();

        let root_url = qr.url.strip_suffix("/mobile.eleve.html").unwrap();

        Self::new(root_url, username, token, LoginMode::QrCode, uuid, None).await
    }

    async fn new(
        root_url: &str,
        username: &str,
        password: &str,
        login_mode: LoginMode,
        uuid: &str,
        nav_identifier: Option<&str>,
    ) -> Result<Client, Error> {
        if username.is_empty() || password.is_empty() {
            return Err(Error::InvalidCredentials);
        }
        let mut communication = Communication::new(&root_url)?;
        // send first request to initialise session and get
        // instance-specific parameters (Params)
        let mut params = communication.initialise(login_mode, nav_identifier).await?;
        // if sent a request with identifiantNav: null, the
        // server returns a new one (fingerprint) to store
        let nav_id = nav_identifier.map(str::to_owned).unwrap_or_else(|| {
            params["dataSec"]["data"]["identifiantNav"]
                .as_str()
                .unwrap()
                .to_owned()
        });
        let params: Params = from_value(params["dataSec"]["data"]["General"].take()).unwrap();
        if params.first_week_day != 2 {
            // 1 - sunday, 2 - monday
            // didn't test on first week day monday so better fail early
            return Err(Error::InvalidStartPage);
        }

        let mut c = Client {
            username: username.to_owned(),
            password: password.to_owned(),
            communication,
            parameters: params,
            user_parameters: None,
            nav_identifier: nav_id,
            next_token: String::new(), // temp, initialized in login
            login_mode,
            uuid: uuid.to_owned(),
        };
        c.login().await?;

        Ok(c)
    }

    pub async fn mock_params() -> Result<Params, Error> {
        let mut params = Value::from_str(include_str!("../resp/params.json")).unwrap();
        Ok(from_value(params["dataSec"]["data"]["General"].take()).unwrap())
    }

    pub async fn mock_user_params() -> Result<UserParams, Error> {
        let mut res = Value::from_str(include_str!("../resp/userparams.json")).unwrap();
        let mut user_parameters: UserParams = from_value(res["dataSec"]["data"].take()).unwrap();
        user_parameters.notifications = parse_notifications(&res["dataSec"]["Signature"]);
        user_parameters.image = res["dataNonSec"]["fichiers"][0]
            .as_str()
            .unwrap()
            .replace("\r\n", ""); // windows :(

        Ok(user_parameters)
    }

    async fn login(&mut self) -> Result<(), Error> {
        let is_qr = self.login_mode == LoginMode::QrCode;
        let is_token = self.login_mode == LoginMode::Token;

        // specify login mode, etc and request challenge
        let mut identification = self
            .communication
            .post(
                "Identification",
                json!({
                    "data": {
                        "genreConnexion": 0,
                        "genreEspace": self.communication.space_id,
                        "identifiant": self.username,
                        "pourENT": false,
                        "enConnexionAuto": false,
                        "demandeConnexionAuto": false,
                        "demandeConnexionAppliMobile": is_qr,
                        "demandeConnexionAppliMobileJeton": is_qr,
                        "enConnexionAppliMobile": is_token,
                        "uuidAppliMobile": self.uuid,
                        "loginTokenSAV": ""
                    }
                }),
            )
            .await?;
        let data = identification["dataSec"]["data"].take();
        let nonce = data.get("alea").and_then(Value::as_str).unwrap_or(""); // usually "{...}", random
        let challenge = data["challenge"].as_str().unwrap();
        let mut auth_username = self.username.clone();
        let mut auth_password = self.password.clone();

        if data["modeCompLog"].as_i64() == Some(1) {
            auth_username = auth_username.to_lowercase();
        } // usually 1 (the username can be uppercase, doesn't matter)
        if data["modeCompMdp"].as_i64() == Some(1) {
            auth_password = auth_password.to_lowercase();
        } // usually 0 (the password should respect upper/lowercase)

        // derive a password by hashing the username, password and nonce
        let auth_key = Encryption::authentication_key(&auth_username, &auth_password, nonce);
        // clone the crypto object (we need to keep the iv)
        // change the key to the one derived from password, etc
        let mut auth_crypto = self.communication.encryption.clone();
        auth_crypto.set_key(&auth_key);
        // re-encrypt the challenge (do not decode hex)
        let challenge = auth_crypto.encrypt(challenge.as_bytes());

        // connexion: 0 = classic mode (there are other ones, eg. from classroom)
        // send the encrypted challenge to the server
        let mut authentication = self
            .communication
            .post(
                "Authentification",
                json!({
                    "data": {
                        "connexion": 0,
                        "challenge": hex::encode(challenge),
                        "espace": self.communication.space_id,
                    }
                }),
            )
            .await?;

        // Acces: 1 = invalid credentials
        // there are also other ones like account banned, etc
        if let Some(_) = authentication.pointer("/dataSec/data/Acces") {
            return Err(Error::InvalidCredentials);
        }

        // each time we log in, save the next token for token login
        if self.login_mode != LoginMode::Password {
            self.next_token = authentication["dataSec"]["data"]["jetonConnexionAppliMobile"]
                .as_str()
                .unwrap()
                .to_owned();
        }

        // update the key used for signing the request numbers
        let data = authentication["dataSec"]["data"].take();
        let key = data["cle"].as_str().unwrap();
        // "5B3235..." (hex) -> "255,0,1,2,..." (32 values) (string) -> 0xDEADBEEF (bytes)
        let key = hex::decode(key).unwrap();
        let key = auth_crypto.decrypt(&key);
        let key = parse_key(&key);
        self.communication.encryption.set_key(&key);

        // get user-specific parameters (UserParams)
        let mut res = self
            .communication
            .post("ParametresUtilisateur", json!({}))
            .await?;
        self.user_parameters = from_value(res["dataSec"]["data"].take()).unwrap();
        self.user_parameters.as_mut().unwrap().notifications =
            parse_notifications(&res["dataSec"]["Signature"]);
        self.user_parameters.as_mut().unwrap().image = res["dataNonSec"]["fichiers"][0]
            .as_str()
            .unwrap()
            .replace("\r\n", ""); // windows :(
        self.parameters.last_week = self.get_week(&self.parameters.last_day);
        // first week returned by server but seems to always be 1

        Ok(())
    }

    pub async fn post(&mut self, function: &str, tab: u64, data: Value) -> Result<Value, Error> {
        if !self
            .user_parameters
            .as_ref()
            .unwrap()
            .allowed_tabs
            .contains(&tab)
        {
            return Err(Error::UnauthorizedTab(tab));
        }
        // to match the website's behaviour, we should send a "Navigation" request
        // each time the tab changes but it just adds unnecessary latency ig
        self.communication
            .post(
                function,
                json!({
                    "Signature": {
                        "onglet": tab
                    },
                    "data": data
                }),
            )
            .await
    }

    pub async fn homework(
        &mut self,
        first_week: u8,
        last_week: u8,
    ) -> Result<Vec<Homework>, Error> {
        if first_week > last_week || last_week > self.parameters.last_week || first_week < 1 {
            return Err(Error::InvalidAction);
        }
        let mut res: Vec<Homework> = from_value(
            self.post(
                "PageCahierDeTexte",
                88,
                json!({
                    "domaine": {
                        "_T": 8,
                        "V": format!("[{}..{}]", first_week, last_week)
                        // for one week, [{week}] works too apparently
                    }
                }),
            )
            .await?["dataSec"]["data"]["ListeTravauxAFaire"]["V"]
                .take(),
        )
        .unwrap();
        for h in &mut res {
            h.order = date_int(&h.due_date);
            for a in &mut h.attachments {
                if a.link.is_empty() {
                    a.link = self.get_attachment(a, false);
                }
            }
        }
        res.sort_by_key(|x| x.order);
        Ok(res)
    }

    pub async fn mock_homework() -> Result<Vec<Homework>, Error> {
        let mut res: Vec<Homework> =
            from_value(
                Value::from_str(include_str!("../resp/homework.json")).unwrap()["dataSec"]["data"]
                    ["ListeTravauxAFaire"]["V"]
                    .take(),
            )
            .unwrap();
        for h in &mut res {
            h.order = date_int(&h.due_date);
            for a in &mut h.attachments {
                if a.link.is_empty() {
                    a.link = "https://example.com/".to_owned()
                }
            }
        }
        res.sort_by_key(|x| x.order);
        Ok(res)
    }

    pub async fn timetable(&mut self, week: u8) -> Result<Vec<Vec<Lesson>>, Error> {
        let resource = self
            .user_parameters
            .as_ref()
            .unwrap()
            .student
            .person
            .serialize();
        let res: Vec<Lesson> = from_value(
            self.post(
                "PageEmploiDuTemps",
                16,
                json!({
                    "ressource": resource,
                    "Ressource": resource,
                    "numeroSemaine": week,
                    "NumeroSemaine": week,
                    "avecAbsencesEleve": false,
                    "avecConseilDeClasse": true,
                    "estEDTPermanence": false,
                    "avecAbsencesRessource": true,
                    "avecDisponibilites": true,
                    "avecInfosPrefsGrille": true
                }),
            )
            .await?["dataSec"]["data"]["ListeCours"]
                .take(),
        )
        .unwrap();
        let mut days = vec![Vec::<Lesson>::new(); self.parameters.week_days as usize];
        for l in res {
            days[l.index as usize / (self.parameters.end_times.len() - 1)].push(l);
        }
        for d in &mut days {
            parse_lessons(d, &self.parameters);
        }

        Ok(days)
    }

    pub async fn mock_timetable(p: &Params) -> Result<Vec<Vec<Lesson>>, Error> {
        let res = Value::from_str(include_str!("../resp/edt.json"))
            .unwrap()["dataSec"]["data"]["ListeCours"].take();
        let res: Vec<Lesson> = from_value(res).unwrap();
        let mut days = vec![Vec::<Lesson>::new(); p.week_days as usize];
        for l in res {
            days[l.index as usize / (p.end_times.len() - 1)].push(l);
        }
        for d in &mut days {
            parse_lessons(d, p);
        }

        Ok(days)
    }

    pub async fn home(&mut self, date: &str) -> Result<Home, Error> {
        let day = json!({
            "_T": 7,
            "V": date.to_owned() + " 0:0:0",
        });
        let full_day = json!({ "date": day });
        let week = self.get_week(date);
        let full_week = json!({ "numeroSemaine": week });

        let mut res = self
            .post(
                "PageAccueil",
                7,
                json!({
                    "avecConseilDeClasse": true,
                    "coursNonAssures": full_week,
                    "dateGrille": day,
                    "donneesProfs": full_week,
                    "donneesVS": full_week,
                    "EDT": full_week,
                    "exclusions": full_week,
                    "incidents": full_week,
                    "menuDeLaCantine": full_day,
                    "modificationsEDT": full_day,
                    "numeroSemaine": week,
                    "partenaireCDI": {"CDI": {}},
                    "personnelsAbsents": full_week,
                    "previsionnelAbsServiceAnnexe": full_day,
                    "registreAppel": full_day,
                    "tableauDeBord": full_day,
                    "TAFARendre": full_day,
                    "TAFEtActivites": full_day,
                }),
            )
            .await?;
        let mut data = res["dataSec"]["data"].take();

        let raw_lessons: Vec<Lesson> = from_value(data["ListeCours"].take()).unwrap();
        // before from_value takes ownership of data (serde doesn't parse ListeCours)

        let mut home: Home = from_value(data).unwrap();
        home.grades.grades.sort_by_key(|x| date_int(&x.date));

        let textbook_icons: Vec<String> = parse_attachments(&res["dataNonSec"]);
        for (i, icon) in textbook_icons.into_iter().enumerate() {
            home.digital_textbooks[i].icon = icon;
        }

        let day_len = (self.parameters.end_times.len() - 1) as u32;
        let today_index = home.first_index / day_len;
        let (mut today, mut next): (Vec<Lesson>, Vec<Lesson>) = raw_lessons
            .into_iter()
            .partition(|l| l.index / day_len == today_index);

        parse_lessons(&mut today, &self.parameters);
        parse_lessons(&mut next, &self.parameters);
        home.lessons = today;
        home.next_lessons = next;

        for h in &mut home.homework {
            for a in &mut h.attachments {
                if a.link.is_empty() {
                    a.link = self.get_attachment(a, false);
                }
            }
        }

        for i in &mut home.attachments.attachments {
            if i.attachment.link.is_empty() {
                i.attachment.link = self.get_attachment(&mut i.attachment, false);
            }
        }

        Ok(home)
    }

    pub async fn mock_home(p: &Params) -> Result<Home, Error> {
        let mut res = Value::from_str(include_str!("../resp/home.json")).unwrap();
        let mut data = res["dataSec"]["data"].take();

        let raw_lessons: Vec<Lesson> = from_value(data["ListeCours"].take()).unwrap();
        // before from_value takes ownership of data (serde doesn't parse ListeCours)

        let mut home: Home = from_value(data).unwrap();
        home.grades.grades.sort_by_key(|x| date_int(&x.date));

        let textbook_icons: Vec<String> = parse_attachments(&res["dataNonSec"]);
        for (i, icon) in textbook_icons.into_iter().enumerate() {
            home.digital_textbooks[i].icon = icon;
        }

        let day_len = (p.end_times.len() - 1) as u32;
        let today_index = home.first_index / day_len;
        let (mut today, mut next): (Vec<Lesson>, Vec<Lesson>) = raw_lessons
            .into_iter()
            .partition(|l| l.index / day_len == today_index);

        parse_lessons(&mut today, p);
        parse_lessons(&mut next, p);
        home.lessons = today;
        home.next_lessons = next;

        for h in &mut home.homework {
            for a in &mut h.attachments {
                if a.link.is_empty() {
                    a.link = "https://example.com/".to_owned()
                }
            }
        }

        for i in &mut home.attachments.attachments {
            if i.attachment.link.is_empty() {
                i.attachment.link = "https://example.com/".to_owned()
            }
        }

        Ok(home)
    }

    pub async fn grades(&mut self, p: &Period) -> Result<Grades, Error> {
        let period_id = &p.id;
        let mut res = self
            .post(
                "DernieresNotes",
                198,
                json!({
                    "Periode": p.serialize(),
                }),
            )
            .await?;
        let mut data: Grades = from_value(res["dataSec"]["data"].take()).unwrap();
        data.grades.retain(|x| &x.period.id == period_id);
        data.grades.sort_by_key(|x| date_int(&x.date));

        for g in &mut data.grades {
            if let Some(s) = &mut g.solution {
                if s.link.is_empty() {
                    s.link = self.get_attachment(&s, true);
                }
            }
        }

        Ok(data)
    }

    pub async fn mock_grades(p: &Period) -> Result<Grades, Error> {
        let mut res = Value::from_str(include_str!("../resp/grades.json")).unwrap();
        let mut data: Grades = from_value(res["dataSec"]["data"].take()).unwrap();
        data.grades.retain(|x| x.period.id == p.id);
        data.grades.sort_by_key(|x| date_int(&x.date));

        for g in &mut data.grades {
            if let Some(s) = &mut g.solution {
                if s.link.is_empty() {
                    s.link = "https://example.com/".to_owned()
                }
            }
        }

        Ok(data)
    }

    pub async fn documents(&mut self, first_week: u8, last_week: u8) -> Result<Resources, Error> {
        if first_week > last_week || last_week > self.parameters.last_week || first_week < 1 {
            return Err(Error::InvalidAction);
        }
        let mut res = self
            .post(
                "PageCahierDeTexte",
                88,
                json!({
                    "domaine": {
                        "_T": 8,
                        "V": format!("[{}..{}]", first_week, last_week)
                        // for one week, [{week}] works too apparently
                    }
                }),
            )
            .await?;
        let mut data: Resources = from_value(res["dataSec"]["data"].take()).unwrap();
        let textbook_icons: Vec<String> = from_value(res["dataNonSec"]["fichiers"].take()).unwrap(); // TODO: could panic?
        for (i, icon) in textbook_icons.into_iter().enumerate() {
            data.digital_textbooks[i].icon = icon;
        }

        for d in &mut data.lesson_details {
            for i in &mut d.items {
                for a in &mut i.attachments {
                    if a.link.is_empty() {
                        a.link = self.get_attachment(a, false);
                    }
                }
            }
        }

        Ok(data)
    }

    pub async fn mock_documents() -> Result<Resources, Error> {
        let mut res = Value::from_str(include_str!("../resp/cont_ress.json")).unwrap();
        let mut data: Resources = from_value(res["dataSec"]["data"].take()).unwrap();
        let textbook_icons: Vec<String> = from_value(res["dataNonSec"]["fichiers"].take()).unwrap(); // TODO: could panic?
        for (i, icon) in textbook_icons.into_iter().enumerate() {
            data.digital_textbooks[i].icon = icon;
        }

        for d in &mut data.lesson_details {
            for i in &mut d.items {
                for a in &mut i.attachments {
                    if a.link.is_empty() {
                        a.link = "https://example.com/".to_owned()
                    }
                }
            }
        }

        Ok(data)
    }

    pub async fn agenda(&mut self) -> Result<Agenda, Error> {
        let mut res = self
            .post(
                "PageAgenda",
                9,
                json!({
                  "avecEventsPasses": true,
                  "listeFamillesFiltre": null,
                  "uniquementMesEvenements": false,
                  "avecRdvPartages": true,
                  "AvecListeClasses": true
                }),
            )
            .await?; // filter old events client-side
        let data: Agenda = from_value(res["dataSec"]["data"].take()).unwrap();

        Ok(data)
    }

    pub async fn mock_agenda() -> Result<Agenda, Error> {
        let mut res = Value::from_str(include_str!("../resp/agenda.json")).unwrap();
        let data: Agenda = from_value(res["dataSec"]["data"].take()).unwrap();

        Ok(data)
    }

    pub async fn news(&mut self) -> Result<News, Error> {
        let mut res = self
            .post(
                "PageActualites",
                8,
                json!({
                    "genreRequeteActualite": 0,
                    "modesAffActus": { "_T": 26, "V": "[0]" }
                }),
            )
            .await?;
        let data: News = from_value(res["dataSec"]["data"].take()).unwrap();

        Ok(data)
    }

    pub async fn mock_news() -> Result<News, Error> {
        let mut res = Value::from_str(include_str!("../resp/info_polls.json")).unwrap();
        let data: News = from_value(res["dataSec"]["data"].take()).unwrap();

        Ok(data)
    }

    pub async fn discussions(&mut self) -> Result<Discussions, Error> {
        let mut res = self
            .post(
                "ListeMessagerie",
                131,
                json!({
                    "avecMessage": true,
                    "avecLu": true
                }),
            )
            .await?;
        let mut data: Discussions = from_value(res["dataSec"]["data"].take()).unwrap();
        data.discussions.retain(|d| d.is_discussion && d.depth == 0);
        // maybe one day we'll parse the threads stuff

        Ok(data)
    }

    pub async fn mock_discussions() -> Result<Discussions, Error> {
        let mut res = Value::from_str(include_str!("../resp/discussions.json")).unwrap();
        let mut data: Discussions = from_value(res["dataSec"]["data"].take()).unwrap();
        data.discussions.retain(|d| d.is_discussion && d.depth == 0);
        // maybe one day we'll parse the threads stuff

        Ok(data)
    }

    pub async fn messages(&mut self, possessions: &[Possession]) -> Result<Messages, Error> {
        let mut res = self
            .post(
                "ListeMessages",
                131,
                json!({
                    "marquerCommeLu": true,
                    "nbMessagesVus": 20,
                    "estNonPossede": false,
                    "listePossessionsMessages": possessions,
                }),
            )
            .await?;
        // actually, idk if nbMessagesVus is always 20
        // also, ["message"]["N"] might always be 0
        // pronotepy simply only sends listePossessionsMessages and it seems to work
        let data: Messages = from_value(res["dataSec"]["data"].take()).unwrap();

        Ok(data)
    }

    pub async fn mock_messages() -> Result<Messages, Error> {
        let mut res = Value::from_str(include_str!("../resp/discussion.json")).unwrap();
        let data: Messages = from_value(res["dataSec"]["data"].take()).unwrap();

        Ok(data)
    }

    pub async fn news_item_content(&mut self, news_item_id: &str) -> Result<Vec<Question>, Error> {
        let person = &self.user_parameters.as_ref().unwrap().student.person;
        let mut res = self
            .post(
                "PageActualites",
                8,
                json!({
                    "genreRequeteActualite": 1,
                    "modeAffActu": 0,
                    "actualite": {
                        "N": news_item_id,
                        "public": person.serialize(),
                        "genrePublic": person.group,
                        "listeCategories": [],
                    }
                }),
            )
            .await?;
        let data: Vec<Question> =
            from_value(res["dataSec"]["data"]["detailsActualite"]["listeQuestions"]["V"].take())
                .unwrap();
        // + "genreRequeteActualite": 1 (like in request)

        Ok(data)
    }

    pub async fn mock_news_item_content() -> Result<Vec<Question>, Error> {
        let mut res = Value::from_str(include_str!("../resp/info.json")).unwrap();
        let data: Vec<Question> =
            from_value(res["dataSec"]["data"]["detailsActualite"]["listeQuestions"]["V"].take())
                .unwrap();
        // + "genreRequeteActualite": 1

        Ok(data)
    }

    pub async fn staff(&mut self) -> Result<Vec<StaffTeacher>, Error> {
        let mut res = self.post("PageEquipePedagogique", 37, json!({})).await?;
        let data: Vec<StaffTeacher> =
            from_value(res["dataSec"]["data"]["liste"]["V"].take()).unwrap();

        Ok(data)
    }

    pub async fn mock_staff() -> Result<Vec<StaffTeacher>, Error> {
        let mut res = Value::from_str(include_str!("../resp/staff.json"))
            .unwrap()
            .clone();
        let data: Vec<StaffTeacher> =
            from_value(res["dataSec"]["data"]["liste"]["V"].take()).unwrap();

        Ok(data)
    }

    pub async fn geolocation(lat: f64, long: f64) -> Result<Vec<School>, Error> {
        let data = json!({
            "nomFonction": "geoLoc",
            "lat": lat.to_string(),
            "long": long.to_string(),
        });

        // got this from pawnote
        let response = reqwest::Client::new()
            .post("https://www.index-education.com/swie/geoloc.php")
            .form(&[("data", data.to_string())])
            .send()
            .await
            .map_err(|_| Error::ConnectionFailed)?
            .error_for_status()
            .map_err(|_| Error::ConnectionFailed)?;

        let body = response.text().await.map_err(|_| Error::ConnectionFailed)?;
        let res = Value::from_str(&body).unwrap();
        let mut data: Vec<School> = from_value(res).unwrap();
        data.retain(|x| !x.url.contains("pronoteprimaire"));

        Ok(data)
    }

    pub async fn mock_geolocation() -> Result<Vec<School>, Error> {
        let res = Value::from_str(include_str!("../resp/geolocation.json")).unwrap();
        let mut data: Vec<School> = from_value(res).unwrap();
        data.retain(|x| !x.url.contains("pronoteprimaire"));

        Ok(data)
    }

    pub async fn set_done(&mut self, homework_id: &str, done: bool) -> Result<(), Error> {
        self.post(
            "SaisieTAFFaitEleve",
            7,
            json!({
                "listeTAF": [{
                    "N": homework_id,
                    "G": 0,
                    "E": 2,
                    "TAFFait": done
                }]
            }),
        )
        .await
        .map(|_| ())
    }

    pub fn get_attachment(&self, a: &Attachment, is_solution: bool) -> String {
        a.get_link(
            &self.communication.encryption,
            self.communication.session_id,
            &self.communication.root_url,
            is_solution,
        )
    }

    pub fn get_week(&self, date: &str) -> u8 {
        // follows pronotepy behaviour
        ((parse_date(date) - parse_date(&self.parameters.first_day)).num_days() / 7 + 1) as u8
    }
}

fn date_int(data: &str) -> u32 {
    let b = data.as_bytes();
    u32::from_str(&format!(
        "{}{}{}",
        std::str::from_utf8(&b[6..10]).unwrap(),
        std::str::from_utf8(&b[3..5]).unwrap(),
        std::str::from_utf8(&b[0..2]).unwrap(),
    ))
    .unwrap()
}

pub fn parse_key(data: &[u8]) -> Vec<u8> {
    let text = std::str::from_utf8(data).unwrap();
    text.split(',')
        .map(|value| value.parse::<u8>().unwrap())
        .collect()
}

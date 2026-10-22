use serde_json::{Value, from_value, json};
use std::str::FromStr;
use std::time::{Duration, Instant};

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
    pub nav_identifier: String,              // unused for now, keep for QR code identification, etc
}

impl Client {
    pub async fn new(url: String, username: String, password: String) -> Result<Client, Error> {
        if username.is_empty() || password.is_empty() {
            return Err(Error::InvalidCredentials);
        }
        let mut communication = Communication::new(&url)?;
        let params = communication.initialise().await?;
        let nav_id = params["dataSec"]["data"]["identifiantNav"]
            .as_str()
            .unwrap()
            .to_owned();
        let params: Params = from_value(params["dataSec"]["data"]["General"].clone()).unwrap();
        if params.first_week_day != 2 {
            // NOT MONDAY :(
            return Err(Error::InvalidStartPage);
        }

        let mut c = Client {
            username: username.to_lowercase(),
            password,
            communication,
            parameters: params,
            user_parameters: None,
            nav_identifier: nav_id,
        };
        c.login().await?;

        Ok(c)
    }

    pub async fn mock_params() -> Result<Params, Error> {
        let params = Value::from_str(include_str!("../resp/params.json")).unwrap();
        Ok(from_value(params["dataSec"]["data"]["General"].clone()).unwrap())
    }

    pub async fn mock_user_params() -> Result<UserParams, Error> {
        let params = Value::from_str(include_str!("../resp/userparams.json")).unwrap();
        Ok(from_value(params["dataSec"]["data"].clone()).unwrap())
    }

    async fn login(&mut self) -> Result<(), Error> {
        let identification = self.identification().await?; // request challenge
        let data = identification["dataSec"]["data"].clone();
        let nonce = data
            .get("alea")
            .and_then(Value::as_str)
            .unwrap_or("".into());
        let challenge = data["challenge"].as_str().unwrap();
        let auth_key = Encryption::authentication_key(&self.username, &self.password, nonce);
        let mut crypto = self.communication.encryption.clone(); // temporary change key
        crypto.key = auth_key;
        let challenge = crypto.encrypt(challenge.as_bytes());

        let authentication = self
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

        if let Some(_) = authentication.pointer("/dataSec/data/Acces") {
            return Err(Error::InvalidCredentials);
        }

        self.finish_authentication(&authentication, auth_key)?;
        let res = self
            .communication
            .post("ParametresUtilisateur", json!({}))
            .await?;
        self.user_parameters = from_value(res["dataSec"]["data"].clone()).unwrap();
        self.user_parameters.as_mut().unwrap().image = res["dataNonSec"]["fichiers"][0]
            .as_str()
            .unwrap()
            .replace("\r\n", ""); // windows :(
        self.parameters.last_week = self.get_week(self.parameters.last_day.clone());
        // first week returned by server but seems to always be 1

        Ok(())
    }

    async fn identification(&mut self) -> Result<Value, Error> {
        self.communication
            .post(
                "Identification",
                json!({
                    "data": {
                        "genreConnexion": 0,
                        "genreEspace": 3,
                        "identifiant": self.username,
                        "pourENT": false,
                        "enConnexionAuto": false,
                        "demandeConnexionAuto": false,
                        "enConnexionAppliMobile": false,
                        "demandeConnexionAppliMobile": false,
                        "demandeConnexionAppliMobileJeton": false,
                        "informationsAppareil": null,
                        "uuidAppliMobile": "",
                        "loginTokenSAV": "",
                    }
                }),
            )
            .await
    }

    fn finish_authentication(&mut self, response: &Value, auth_key: [u8; 16]) -> Result<(), Error> {
        let data = response["dataSec"]["data"].clone();
        let key = data["cle"].as_str().unwrap();
        let mut crypto = self.communication.encryption.clone();
        crypto.key = auth_key;
        // "5B3235..." (hex) -> "[255, 0, 1, 2]" (string) -> 0xDEADBEEF (bytes)
        let key = hex::decode(key).map_err(|_| Error::InvalidResponse)?;
        let key = crypto.decrypt(&key)?;
        let key = parse_key(&key)?;
        self.communication.encryption.set_key(&key);
        Ok(())
    }

    pub async fn post(&mut self, function: &str, tab: u64, data: Value) -> Result<Value, Error> {
        if !self
            .user_parameters
            .clone()
            .unwrap()
            .allowed_tabs
            .contains(&tab)
        {
            return Err(Error::UnauthorizedTab(tab));
        }
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
                .clone(),
        )
        .unwrap();
        for h in &mut res {
            h.order = date_int(&h.due_date);
            for a in &mut h.attachments {
                if a.link.is_empty() {
                    a.link = self.get_attachment(a);
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
                    .clone(),
            )
            .unwrap();
        for h in &mut res {
            h.order = date_int(&h.due_date)
        }
        res.sort_by_key(|x| x.order);
        Ok(res)
    }

    pub async fn timetable(&mut self, week: u8) -> Result<Vec<Vec<Lesson>>, Error> {
        let resource = self.user_parameters.clone().unwrap().resource.serialize();
        let mut res: Vec<Lesson> = from_value(
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
                .clone(),
        )
        .unwrap();
        let mut days = vec![Vec::<Lesson>::new(); self.parameters.week_days as usize];
        for l in &mut res {
            days[l.index as usize / (self.parameters.end_times.len() - 1)].push(l.clone());
        }
        for d in &mut days {
            parse_lessons(d, &self.parameters);
        }

        Ok(days)
    }

    pub async fn mock_timetable(p: &Params) -> Result<Vec<Vec<Lesson>>, Error> {
        let res = Value::from_str(include_str!("../resp/edt.json"))
            .unwrap()["dataSec"]["data"]["ListeCours"]
            .clone();
        let mut res: Vec<Lesson> = from_value(res).unwrap();
        let mut days = vec![Vec::<Lesson>::new(); p.week_days as usize];
        for l in &mut res {
            days[l.index as usize / (p.end_times.len() - 1)].push(l.clone());
        }
        for d in &mut days {
            parse_lessons(d, p);
        }

        Ok(days)
    }

    pub async fn home(&mut self, date: String) -> Result<Home, Error> {
        let day = json!({
            "_T": 7,
            "V": date.clone() + " 0:0:0",
        });
        let full_day = json!({ "date": day });
        let week = self.get_week(date);
        let full_week = json!({ "numeroSemaine": week });

        let res = self
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
        let data = &res["dataSec"]["data"];
        let mut home: Home = from_value(data.clone()).unwrap();
        home.grades.grades.sort_by_key(|x| date_int(&x.date));

        let textbook_icons: Vec<String> = parse_attachments(res["dataNonSec"].clone());
        for (i, icon) in textbook_icons.into_iter().enumerate() {
            home.digital_textbooks[i].icon = icon;
        }

        let raw_lessons: Vec<Lesson> = from_value(data["ListeCours"].clone()).unwrap();

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
                    a.link = self.get_attachment(a);
                }
            }
        }

        Ok(home)
    }

    pub async fn mock_home(p: &Params) -> Result<Home, Error> {
        let res = Value::from_str(include_str!("../resp/home.json"))
            .unwrap()
            .clone();
        let data = &res["dataSec"]["data"];
        let mut home: Home = from_value(data.clone()).unwrap();
        home.grades.grades.sort_by_key(|x| date_int(&x.date));

        let textbook_icons: Vec<String> = parse_attachments(res["dataNonSec"].clone());
        for (i, icon) in textbook_icons.into_iter().enumerate() {
            home.digital_textbooks[i].icon = icon;
        }

        let raw_lessons: Vec<Lesson> = from_value(data["ListeCours"].clone()).unwrap();

        let day_len = (p.end_times.len() - 1) as u32;
        let today_index = home.first_index / day_len;
        let (mut today, mut next): (Vec<Lesson>, Vec<Lesson>) = raw_lessons
            .into_iter()
            .partition(|l| l.index / day_len == today_index);

        parse_lessons(&mut today, p);
        parse_lessons(&mut next, p);
        home.lessons = today;
        home.next_lessons = next;

        Ok(home)
    }

    pub async fn grades(&mut self, p: Period) -> Result<Grades, Error> {
        let period_id = p.id.clone();
        let res = self
            .post(
                "DernieresNotes",
                198,
                json!({
                    "Periode": p.serialize(),
                }),
            )
            .await?;
        let mut data: Grades = from_value(res["dataSec"]["data"].clone()).unwrap();
        data.grades.retain(|x| x.period.id == period_id);
        data.grades.sort_by_key(|x| date_int(&x.date));

        Ok(data)
    }

    pub async fn mock_grades(p: Period) -> Result<Grades, Error> {
        let res = Value::from_str(include_str!("../resp/grades.json"))
            .unwrap()
            .clone();
        let mut data: Grades = from_value(res["dataSec"]["data"].clone()).unwrap();
        data.grades.retain(|x| x.period.id == p.id);
        data.grades.sort_by_key(|x| date_int(&x.date));

        Ok(data)
    }

    pub async fn documents(&mut self, first_week: u8, last_week: u8) -> Result<Resources, Error> {
        if first_week > last_week || last_week > self.parameters.last_week || first_week < 1 {
            return Err(Error::InvalidAction);
        }
        let res = self
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
        let mut data: Resources = from_value(res["dataSec"]["data"].clone()).unwrap();
        let textbook_icons: Vec<String> =
            from_value(res["dataNonSec"]["fichiers"].clone()).unwrap(); // TODO: could panic?
        for (i, icon) in textbook_icons.into_iter().enumerate() {
            data.digital_textbooks[i].icon = icon;
        }

        for d in &mut data.lesson_details {
            for i in &mut d.items {
                for a in &mut i.attachments {
                    if a.link.is_empty() {
                        a.link = self.get_attachment(a);
                    }
                }
            }
        }

        Ok(data)
    }

    pub async fn mock_documents() -> Result<Resources, Error> {
        let res = Value::from_str(include_str!("../resp/cont_ress.json"))
            .unwrap()
            .clone();
        let mut data: Resources = from_value(res["dataSec"]["data"].clone()).unwrap();
        let textbook_icons: Vec<String> =
            from_value(res["dataNonSec"]["fichiers"].clone()).unwrap(); // TODO: could panic?
        for (i, icon) in textbook_icons.into_iter().enumerate() {
            data.digital_textbooks[i].icon = icon;
        }

        Ok(data)
    }

    pub async fn agenda(&mut self) -> Result<Agenda, Error> {
        let res = self.post("PageAgenda", 9, json!({
          "avecEventsPasses": true,
          "listeFamillesFiltre": null,
          "uniquementMesEvenements": false,
          "avecRdvPartages": true,
          "AvecListeClasses": true
        })).await?; // filter old events client-side
        let data: Agenda = from_value(res["dataSec"]["data"].clone()).unwrap();

        Ok(data)
    }

    pub async fn mock_agenda() -> Result<Agenda, Error> {
        let res = Value::from_str(include_str!("../resp/agenda.json"))
            .unwrap()
            .clone();
        let data: Agenda = from_value(res["dataSec"]["data"].clone()).unwrap();

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

    pub fn get_attachment(&self, a: &Attachment) -> String {
        a.get_link(
            &self.communication.encryption,
            self.communication.session_id,
            &self.communication.root_url,
        )
    }

    pub async fn keep_alive(&mut self) -> () {
        loop {
            if Instant::now() - self.communication.last_ping > Duration::from_secs(110) {
                let _ = self.post("Navigation", 7, json!({"onglet": 7, "ongletPrec": 7}));
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    pub fn get_week(&self, date: String) -> u8 {
        // follows pronotepy behaviour
        ((parse_date(date).unwrap() - parse_date(self.parameters.first_day.clone()).unwrap())
            .num_days()
            / 7
            + 1) as u8
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

fn parse_key(data: &[u8]) -> Result<Vec<u8>, Error> {
    let text = std::str::from_utf8(data).map_err(|_| Error::InvalidResponse)?;
    text.split(',')
        .map(|value| {
            value
                .trim()
                .parse::<u8>()
                .map_err(|_| Error::InvalidResponse)
        })
        .collect()
}

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
}

impl Period {
    pub fn serialize(self) -> Value {
        json!({
            "G": 2,
            "L": self.name,
            "N": self.id,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FullPeriod {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "dateDebut", deserialize_with = "de_string")]
    pub first_day: String,
    #[serde(rename = "dateFin", deserialize_with = "de_string")]
    pub last_day: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "couleur", default)]
    pub color: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Homework {
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "ordre", default)]
    pub order: u32,
    #[serde(rename = "descriptif", deserialize_with = "de_string")]
    pub description: String,
    #[serde(rename = "TAFFait")]
    pub done: bool,
    #[serde(rename = "niveauDifficulte")]
    pub difficulty: u32,
    #[serde(rename = "duree")]
    pub duration: f32, // minutes
    #[serde(rename = "DonneLe", alias = "donneLe", deserialize_with = "de_date")]
    pub creation_date: String, //NaiveDate,
    #[serde(rename = "PourLe", alias = "pourLe", deserialize_with = "de_date")]
    pub due_date: String, //NaiveDate,
    #[serde(rename = "Matiere", alias = "matiere", deserialize_with = "de_subject")]
    pub subject: Subject,
    #[serde(rename = "CouleurFond")]
    pub color: String, // #XXXXXX format
    #[serde(rename = "nomPublic", default)]
    pub group: Option<String>,
    /* #[serde(rename = "ListeThemes")]
    pub themes: Value, // TODO: parse themes
    #[serde(rename = "ListePieceJointe")]
    pub attachments: Value,  // TODO: parse attachments
    */
    // unused: libelleCBTheme, avecMiseEnForme, genreRendu, peuRendre,
    // avecRendu, avecRendu, cahierDeTextes, cours, avecMiseEnForme
    // QCMFait, executionQCM
}

#[derive(Default, Debug, Clone)]
pub struct LessonContent {
    pub subject: Subject,
    pub teacher: Vec<String>,
    pub classroom: Vec<Classroom>,
    pub group: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Classroom {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
}

// TODO: lesson content
// {
// 	"cahierDeTextes": {
// 		"_T": 24,
// 		"V": {
// 			"N": "ID",
// 			"estDevoir": true,
// 			"estEval": true,
// 			"originesCategorie": {
// 				"_T": 24,
// 				"V": [
// 					{
// 						"G": 3,
// 						"L": "Devoir sur table",
// 						"libelleIcone": "DS"
// 					},
// 					{
//            "G": 7,
//            "L": "Évaluation de compétences",
//            "libelleIcone": "EVA"
//          }
// 				]
// 			}
// 		}
// 	}
// }

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Lesson {
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "P")]
    pub order: u32,
    #[serde(rename = "Statut", default)]
    pub status: String,
    #[serde(rename = "estAnnule", default)]
    pub cancelled: bool,
    #[serde(rename = "place")]
    pub index: u32,
    #[serde(rename = "duree")]
    pub duration: u32,
    #[serde(rename = "DateDuCours", deserialize_with = "de_datetime")]
    pub start: String, //NaiveDateTime
    #[serde(rename = "DateDuCoursFin", default)]
    pub end: String,
    #[serde(rename = "CouleurFond")]
    pub color: String,
    #[serde(rename = "ListeContenus", deserialize_with = "de_lesson_content")]
    pub content: LessonContent,
    #[serde(rename = "memo", default)]
    pub note: String,
    // unused: G, P, AvecTafPublie, cahierDeTextes, avecCdT
}

#[derive(Debug, Clone, Deserialize)]
pub struct Collectivity {
    #[serde(rename = "urlCollectivite", default)]
    pub url: Option<String>,
    #[serde(rename = "libelleCollectivite", default)]
    pub name: Option<String>,
    #[serde(rename = "couleurs", deserialize_with = "de_color", default)]
    pub color: Option<String>, // #xxxxxx
}

#[derive(Debug, Clone, Deserialize)]
pub struct Params {
    #[serde(rename = "PremierLundi", deserialize_with = "de_date")]
    pub first_day: String,
    #[serde(rename = "DerniereDate", deserialize_with = "de_date")]
    pub last_day: String,
    #[serde(rename = "premierJourSemaine")]
    pub first_week_day: u8, // 1 - sunday, 2 - monday, ..., 7 - saturday (???)
    #[serde(rename = "joursOuvresParCycle")]
    pub week_days: u8, // monday - friday: 5
    #[serde(rename = "debutDemiPension")]
    pub lunch_start: u32, // end_times[i - 1]
    #[serde(rename = "finDemiPension")]
    pub lunch_end: u32, // same
    #[serde(rename = "versionPN")]
    pub version: String, // 2026.2.6
    #[serde(rename = "version")]
    pub full_version: String,
    #[serde(rename = "ListePeriodes")]
    pub periods: Vec<FullPeriod>,
    #[serde(skip)]
    pub last_week: u8,
    #[serde(rename = "ListeHeuresFin", deserialize_with = "de_end_times")]
    pub end_times: Vec<EndTime>,
    #[serde(rename = "Collectivite")]
    pub collectivity: Collectivity,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndTime {
    pub time: String,
    pub show: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserParams {
    #[serde(rename = "ressource")]
    pub resource: Resource,
    #[serde(rename = "listeOnglets", deserialize_with = "de_tabs")]
    pub allowed_tabs: Vec<u64>,
    #[serde(skip)]
    pub image: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Resource {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "classeDEleve")]
    pub class: Class,
    #[serde(rename = "listeOngletsPourPeriodes", deserialize_with = "de_periods")]
    pub periods: Periods,
    // listeGroupes
}

impl Resource {
    pub fn serialize(self) -> Value {
        json!({
            "G": 4,
            "L": self.name,
            "N": self.id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Periods {
    pub periods: Vec<Period>,
    pub default: Period,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Home {
    #[serde(rename = "estDansUnePeriodeDeDeconnexion")]
    pub disconnection: bool,
    #[serde(rename = "dateSelectionnee", deserialize_with = "de_date")]
    pub date: String,
    #[serde(rename = "prochaineDate", deserialize_with = "de_date_some", default)]
    pub next_date: Option<String>,
    #[serde(rename = "premierePlaceHebdoDuJour")]
    pub first_index: u32,
    #[serde(skip)]
    pub lessons: Vec<Lesson>,
    #[serde(skip)]
    pub next_lessons: Vec<Lesson>,
    #[serde(rename = "travailAFaire", deserialize_with = "de_homework")]
    pub homework: Vec<Homework>,
    #[serde(rename = "notes", deserialize_with = "de_home_grades")]
    pub grades: HomeGrades,
    #[serde(skip)]
    pub attachments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeGrades {
    pub grades: Vec<Grade>,
    pub period: Period,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grades {
    #[serde(rename = "moyGenerale", deserialize_with = "de_your_grade", default)]
    pub average: GradeValue,
    #[serde(
        rename = "moyGeneraleClasse",
        deserialize_with = "de_your_grade",
        default
    )]
    pub class_average: GradeValue,
    #[serde(rename = "baremeMoyGenerale", deserialize_with = "de_bareme", default)]
    pub average_scale: u32, // can be zero if empty average
    #[serde(
        rename = "baremeMoyGeneraleParDefaut",
        deserialize_with = "de_bareme",
        default
    )]
    pub average_default_scale: u32, // can be zero if empty average
    #[serde(rename = "listeDevoirs", deserialize_with = "de_grades")]
    pub grades: Vec<Grade>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grade {
    #[serde(rename = "note", deserialize_with = "de_your_grade")]
    pub grade: GradeValue,
    #[serde(rename = "noteMin", deserialize_with = "de_your_grade", default)]
    pub min_grade: GradeValue,
    #[serde(rename = "noteMax", deserialize_with = "de_your_grade", default)]
    pub max_grade: GradeValue,
    #[serde(rename = "moyenne", deserialize_with = "de_your_grade", default)]
    pub average: GradeValue,
    #[serde(rename = "bareme", deserialize_with = "de_bareme")]
    pub scale: u32,
    #[serde(rename = "baremeParDefaut", deserialize_with = "de_bareme")]
    pub default_scale: u32,
    #[serde(rename = "date", deserialize_with = "de_date")]
    pub date: String,
    #[serde(rename = "service", deserialize_with = "de_subject")]
    pub subject: Subject,
    #[serde(rename = "commentaire", default)]
    pub description: String,
    #[serde(rename = "coefficient", default)]
    pub weight: Option<f32>,
    #[serde(rename = "estFacultatif", default)]
    pub is_optional: bool,
    #[serde(rename = "estBonus", default)]
    pub is_bonus: bool,
    #[serde(rename = "estEnGroupe", default)]
    pub is_group_work: bool,
    #[serde(rename = "periode", deserialize_with = "de_period")]
    pub period: Period,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum GradeValue {
    Value(f32),
    Absent,
    Excused,
    NotGraded,
    Unfit,
    NotHandedIn,
    AbsentZero,
    NotHandedInZero,
    Congratulations,
    #[default]
    Empty,
}

pub fn pretty_end_time(end_time: u32, l: &Lesson, p: &Params) -> String {
    l.start.split_once(" ").unwrap().0.to_owned()
        + " "
        + &p.end_times[(end_time - 1) as usize]
            .time
            .clone()
            .replace("h", ":")
        + ":00"
}

fn gap(id: &str, name: &str, index: u32, duration: u32, l: &Lesson, p: &Params) -> Lesson {
    Lesson {
        id: id.into(),
        order: 1000,
        status: "".into(),
        cancelled: false,
        index,
        duration,
        start: pretty_end_time(index, l, p),
        end: pretty_end_time(index + duration, l, p),
        color: "".into(),
        content: LessonContent {
            subject: Subject {
                name: name.into(),
                id: id.into(),
                color: None,
            },
            teacher: Vec::new(),
            classroom: Vec::new(),
            group: None,
        },
        note: "".into()
    }
}

pub fn parse_lessons(d: &mut Vec<Lesson>, p: &Params) {
    // empty end -> you must add it (derive from index + duration)
    for x in &mut *d {
        x.index = x.index % (p.end_times.len() - 1) as u32;
        if x.end.is_empty() {
            x.end = pretty_end_time(x.index + x.duration, &x, p);
        }
    }

    // sort by lesson start time
    d.sort_by_key(|x| x.index);

    // resolve overlapping lessons: choose the one with the greatest order
    let mut ids = Vec::new();
    for l in d.windows(2) {
        if l[1].index < l[0].index + l[0].duration {
            ids.push(if l[1].order > l[0].order {
                l[0].id.clone()
            } else {
                l[1].id.clone()
            });
        }
    }
    d.retain(|x| !ids.contains(&x.id));

    // fill gaps + lunch
    let mut gaps = Vec::new();
    for l in d.windows(2) {
        let l0_end = l[0].index + l[0].duration;
        let l1_start = l[1].index;

        // no gap
        if l0_end == l1_start {
            continue;
        }

        // first lesson end / lunch start if it overlaps
        let lunch_start = l0_end.max(p.lunch_start);
        // second lesson start / lunch end if it overlaps
        let lunch_end = l1_start.min(p.lunch_end);

        // if first lesson ends before lunch_start, create a gap until:
        // - second lesson start if it's before lunch start too (normal free time)
        // - lunch start else (free time lunch overlap)
        if l0_end < lunch_start {
            let free_end = l1_start.min(lunch_start);
            gaps.push(gap(
                "free",
                "Pas de cours",
                l0_end,
                free_end - l0_end,
                &d[0],
                p,
            ));
        }

        // only true when gap overlaps lunch period
        if lunch_start < lunch_end {
            gaps.push(gap(
                "lunch",
                "Déjeuner",
                lunch_start,
                lunch_end - lunch_start,
                &d[0],
                p,
            ));
        }

        // if second lesson starts after lunch_end, create a gap from:
        // - lesson end if it's after lunch end too (normal free time)
        // - lunch end else (free time lunch overlap)
        if lunch_end < l1_start {
            let free_start = l0_end.max(lunch_end);
            gaps.push(gap(
                "free",
                "Pas de cours",
                free_start,
                l1_start - free_start,
                &d[0],
                p,
            ));
        }
    }
    // wednesday, we finish with the lunch -> windows(2) doesnt work :(
    if let Some(l) = d.last() {
        // take the end of the last lesson
        let l_end = l.index + l.duration;

        // create a free gap before lunch period when needed
        if l_end < p.lunch_start {
            gaps.push(gap(
                "free",
                "Pas de cours",
                l_end,
                p.lunch_start - l_end,
                &d[0],
                p,
            ));
        }

        // create actual lunch
        if l_end <= p.lunch_start {
            gaps.push(gap(
                "lunch",
                "Déjeuner",
                p.lunch_start,
                p.lunch_end - p.lunch_start,
                &d[0],
                p,
            ));
        }
    }
    d.extend(gaps);

    // sort by lesson start time
    d.sort_by_key(|x| x.index);
}

pub fn parse_attachments(res: Value) -> Vec<String> {
    if let Some(files) = res.get("fichiers") {
        let mut a = Vec::new();
        for r in files.as_array().unwrap() {
            a.push(r.as_str().unwrap().replace("\r\n", ""))
        }
        return a;
    }
    return Vec::new();
}

fn de_grades<'de, D>(deserializer: D) -> Result<Vec<Grade>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?["V"].clone();
    let res: Vec<Grade> = serde_json::from_value(value).unwrap();

    Ok(res)
}

fn de_home_grades<'de, D>(deserializer: D) -> Result<HomeGrades, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;

    let grades: Vec<Grade> = serde_json::from_value(value["listeDevoirs"]["V"].clone()).unwrap();
    let period: Period = serde_json::from_value(value["page"]["periode"]["V"].clone()).unwrap();

    Ok(HomeGrades { grades, period })
}

fn de_period<'de, D>(deserializer: D) -> Result<Period, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?["V"].clone();
    let res: Period = serde_json::from_value(value).unwrap();

    Ok(res)
}

fn de_homework<'de, D>(deserializer: D) -> Result<Vec<Homework>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?["listeTAF"]["V"].clone();
    let mut res: Vec<Homework> = serde_json::from_value(value).unwrap();
    res.sort_by_key(|x| x.order);

    Ok(res)
}

fn de_end_times<'de, D>(deserializer: D) -> Result<Vec<EndTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?["V"].clone();
    let mut res = Vec::new();
    for v in value.as_array().unwrap() {
        res.push(EndTime {
            time: v["L"].as_str().unwrap().into(),
            show: v.get("A").and_then(Value::as_bool).unwrap_or(true),
        })
    }

    Ok(res)
}

fn de_tabs<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(extract_onglets(value))
}

fn de_periods<'de, D>(deserializer: D) -> Result<Periods, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    let tabs = value["V"].as_array().unwrap();
    let entries = tabs
        .iter()
        .find(|entry| entry.get("G").and_then(Value::as_u64) == Some(198))
        // Get periods for the grades tab
        .or_else(|| tabs.first())
        // ... Or fallback to first one (matches pronotepy's behaviour)
        .unwrap();
    let periods = entries["listePeriodes"]["V"].as_array().unwrap().clone();
    let periods: Vec<Period> = periods
        .into_iter()
        .filter(|entry| entry.get("N").is_some()) // Fake period
        .map(|json| serde_json::from_value::<Period>(json).unwrap())
        .collect();
    let default_period: Period =
        serde_json::from_value(entries["periodeParDefaut"]["V"].clone()).unwrap();

    Ok(Periods {
        periods: periods,
        default: default_period,
    })
}

fn de_lesson_content<'de, D>(deserializer: D) -> Result<LessonContent, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values: Vec<serde_json::Value> = Value::deserialize(deserializer)?["V"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut subject: Option<Subject> = None;
    let mut teacher: Vec<String> = Vec::new();
    let mut classroom: Vec<Classroom> = Vec::new();
    let mut group = None;

    for value in values {
        match value["G"].as_u64() {
            Some(16) => {
                subject = Some(serde_json::from_value(value).map_err(serde::de::Error::custom)?);
            }
            Some(3) => {
                if let Some(t) = value["L"].as_str() {
                    teacher.push(t.to_owned());
                }
            }
            Some(2) => {
                group = value["L"].as_str().map(String::from);
            }
            Some(17) => {
                classroom.push(serde_json::from_value(value).map_err(serde::de::Error::custom)?);
            }
            _ => {}
        }
    }

    Ok(LessonContent {
        subject: subject.unwrap(),
        teacher,
        classroom,
        group,
    })
}

fn de_string<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;

    Ok(value["V"].as_str().unwrap().into())
}

fn de_your_grade<'de, D>(d: D) -> Result<GradeValue, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;
    let s = value["V"].as_str().unwrap();

    Ok(match s {
        "" => GradeValue::Empty,
        "|1" => GradeValue::Absent,
        "|2" => GradeValue::Excused,
        "|3" => GradeValue::NotGraded,
        "|4" => GradeValue::Unfit,
        "|5" => GradeValue::NotHandedIn,
        "|6" => GradeValue::AbsentZero,
        "|7" => GradeValue::NotHandedInZero,
        "|8" => GradeValue::Congratulations,
        _ => s.replace(',', ".").parse().map(GradeValue::Value).unwrap(),
    })
}

fn de_bareme<'de, D>(d: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;

    Ok(value["V"].as_str().unwrap().parse().unwrap())
}

fn de_color<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;
    Ok(Some(value["ligne"].as_str().unwrap().into()))
}

fn de_date_some<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;

    Ok(Some(value["V"].as_str().unwrap().into()))
}

fn de_date<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;

    // Ok(parse_date(value["V"].as_str().unwrap().into()).unwrap())
    Ok(value["V"].as_str().unwrap().into())
}

fn de_datetime<'de, D>(d: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;

    // Ok(parse_datetime(value["V"].as_str().unwrap().into()).unwrap())
    Ok(value["V"].as_str().unwrap().into())
}

fn de_subject<'de, D>(d: D) -> Result<Subject, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(d)?;

    Ok(Subject {
        name: value["V"]["L"].as_str().unwrap().into(),
        id: value["V"]["N"].as_str().unwrap().into(),
        color: value["V"]
            .get("couleur")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

pub fn parse_date(date: String) -> Result<NaiveDate, crate::Error> {
    NaiveDate::parse_from_str(&date, "%d/%m/%Y").map_err(|_| crate::Error::InvalidResponse)
}

pub fn parse_datetime(date: String) -> Result<NaiveDateTime, crate::Error> {
    NaiveDateTime::parse_from_str(&date, "%d/%m/%Y %H:%M:%S")
        .map_err(|_| crate::Error::InvalidResponse)
}

fn extract_onglets(value: Value) -> Vec<u64> {
    match value {
        Value::Array(values) => values.into_iter().flat_map(extract_onglets).collect(),
        Value::Object(values) => values.into_values().flat_map(extract_onglets).collect(),
        Value::Number(number) => number.as_u64().into_iter().collect(),
        _ => Vec::new(),
    }
}

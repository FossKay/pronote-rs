use crate::helpers::*;
use serde::{Deserialize, Serialize};
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
pub struct Teacher {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Attachment {
    #[serde(rename = "L")]
    pub file: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "url", default)]
    pub link: String,
    #[serde(rename = "estUnLienInterne", default)] // ?!
    pub interal_link: bool,
}

impl Attachment {
    pub fn get_link(
        &self,
        encryption: &crate::encryption::Encryption,
        session: u64,
        root_url: &str, // no slash at end plz
    ) -> String {
        // {{ escapes the "{" character (in format!)
        let json = format!(
            r#"{{"N":{},"Actif":true}}"#,
            serde_json::to_string(&self.id).unwrap()
        );
        let bytes = json.as_bytes();
        let ciphertext = encryption.encrypt(bytes);
        let hex: String = hex::encode(ciphertext);

        let filename = self
            .file
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#039;", "'")
            .replace("&amp;", "&"); // uri encode?

        format!(
            "{}/FichiersExternes/{}/{}?Session={}",
            root_url, hex, filename, session,
        )
    }
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Subject {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "couleur", default)]
    pub color: String, // empty in most cases
    #[serde(rename = "ordre", default)]
    pub order: u32, // empty in most cases
    #[serde(rename = "moyEleve", deserialize_with = "de_grade", default)]
    pub average: GradeValue,
    #[serde(rename = "moyClasse", deserialize_with = "de_grade", default)]
    pub class_average: GradeValue,
    #[serde(rename = "moyMin", deserialize_with = "de_grade", default)]
    pub min_average: GradeValue,
    #[serde(rename = "moyMax", deserialize_with = "de_grade", default)]
    pub max_average: GradeValue,
    #[serde(rename = "baremeMoyEleve", deserialize_with = "de_bareme", default)]
    pub average_scale: u32,
    #[serde(
        rename = "baremeMoyEleveParDefaut",
        deserialize_with = "de_bareme",
        default
    )]
    pub default_average_scale: u32,
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
    pub themes: Value, // TODO: parse themes */
    #[serde(
        rename = "ListePieceJointe",
        alias = "listeDocumentJoint",
        deserialize_with = "de_attachments"
    )]
    pub attachments: Vec<Attachment>,
    // TODO: peuRendre
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

#[derive(Debug, Clone, Deserialize)]
pub struct LessonDetailsLink {
    #[serde(rename = "N")]
    pub lesson_details_id: String,
    #[serde(rename = "estDevoir", default)]
    pub is_ds: bool,
    #[serde(rename = "estEval", default)]
    pub is_eva: bool,
    #[serde(rename = "originesCategorie", deserialize_with = "de_categories")]
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    #[serde(rename = "L", default)]
    pub name: String, // empty and N=0 if nothing
    #[serde(rename = "N", default)]
    pub id: String, // empty for timetable
    #[serde(rename = "libelleIcone")]
    pub label: String,
}

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
    pub start: String,
    #[serde(rename = "DateDuCoursFin", default)]
    pub end: String,
    #[serde(rename = "CouleurFond")]
    pub color: String,
    #[serde(rename = "ListeContenus", deserialize_with = "de_lesson_content")]
    pub content: LessonContent,
    #[serde(rename = "memo", default)]
    pub note: String,
    #[serde(
        rename = "cahierDeTextes",
        deserialize_with = "de_lesson_details_link",
        default
    )]
    pub lesson_details_link: Option<LessonDetailsLink>,
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
    #[serde(rename = "listeJoursFeries", deserialize_with = "de_holidays")]
    pub holidays: Vec<Holiday>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndTime {
    pub time: String,
    pub show: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Holiday {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "dateDebut", deserialize_with = "de_date")]
    pub start: String,
    #[serde(rename = "dateFin", deserialize_with = "de_date")]
    pub end: String,
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
    #[serde(rename = "listeGroupes", deserialize_with = "de_groups")]
    pub groups: Vec<Class>,
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
    #[serde(rename = "kiosque", deserialize_with = "de_digital_textbooks2")]
    pub digital_textbooks: Vec<DigitalTextbook>,
    #[serde(rename = "agenda", deserialize_with = "de_home_agenda")]
    pub events: Vec<Event>,
    #[serde(rename = "ressourcePedagogique")]
    pub attachments: HomeAttachments,
    #[serde(rename = "penseBete", deserialize_with = "de_sticky_note")]
    pub sticky_note: String,
    #[serde(rename = "devoirSurveille", deserialize_with = "de_exams")]
    pub exams: Vec<Exam>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeAttachments {
    #[serde(rename = "listeRessources", deserialize_with = "de_home_attachments")]
    pub attachments: Vec<HomeAttachment>,
    #[serde(rename = "listeMatieres", deserialize_with = "de_subjects")]
    pub subjects: Vec<Subject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeAttachment {
    #[serde(rename = "ressource", deserialize_with = "de_attachment")]
    pub attachment: Attachment,
    #[serde(rename = "date", deserialize_with = "de_date")]
    pub date: String,
    #[serde(rename = "matiere", deserialize_with = "de_reference_id")]
    pub subject_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeGrades {
    pub grades: Vec<Grade>,
    pub period: Period,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Exam {
    #[serde(rename = "L")]
    pub title: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "matiere", deserialize_with = "de_subject")]
    pub subject: Subject,
    #[serde(rename = "couleur")]
    pub color: String,
    #[serde(rename = "dateDebut", deserialize_with = "de_datetime")]
    pub start: String,
    #[serde(rename = "dateFin", deserialize_with = "de_datetime")]
    pub end: String,
    #[serde(rename = "listeSalles", deserialize_with = "de_classrooms")]
    pub classrooms: Vec<Classroom>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grades {
    #[serde(rename = "moyGenerale", deserialize_with = "de_grade", default)]
    pub average: GradeValue,
    #[serde(rename = "moyGeneraleClasse", deserialize_with = "de_grade", default)]
    pub class_average: GradeValue,
    #[serde(rename = "baremeMoyGenerale", deserialize_with = "de_bareme", default)]
    pub average_scale: u32, // can be zero if empty average
    #[serde(
        rename = "baremeMoyGeneraleParDefaut",
        deserialize_with = "de_bareme",
        default
    )]
    pub average_default_scale: u32, // can be zero if empty average
    #[serde(rename = "avecDetailService")]
    pub subject_averages: bool,
    #[serde(rename = "avecDetailDevoir")]
    pub grade_averages: bool,
    #[serde(rename = "listeServices", deserialize_with = "de_subjects")]
    pub subjects: Vec<Subject>,
    #[serde(rename = "listeDevoirs", deserialize_with = "de_grades")]
    pub grades: Vec<Grade>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grade {
    #[serde(rename = "note", deserialize_with = "de_grade")]
    pub grade: GradeValue,
    #[serde(rename = "noteMin", deserialize_with = "de_grade", default)]
    pub min_grade: GradeValue,
    #[serde(rename = "noteMax", deserialize_with = "de_grade", default)]
    pub max_grade: GradeValue,
    #[serde(rename = "moyenne", deserialize_with = "de_grade", default)]
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
    #[serde(rename = "elmCorrige", deserialize_with = "de_attachment_some", default)]
    pub solution: Option<Attachment>, // there is also libelleCorrige
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum GradeValue {
    // got these from pronotepy
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

#[derive(Debug, Clone, Deserialize)]
pub struct Resources {
    #[serde(rename = "ListeCahierDeTextes", deserialize_with = "de_lesson_details")]
    pub lesson_details: Vec<LessonDetails>,
    /* there is ListeRessourcesPedagogiques but idk seems unnecessary?! */
    #[serde(
        rename = "ListeRessourcesNumeriques",
        deserialize_with = "de_digital_textbooks"
    )]
    pub digital_textbooks: Vec<DigitalTextbook>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DigitalTextbook {
    #[serde(rename = "titre")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "editeur")]
    pub author: String,
    #[serde(rename = "description", deserialize_with = "de_string")]
    pub description: String, // rename not mandatory
    #[serde(skip)]
    pub icon: String, // base64 png
}

#[derive(Debug, Clone, Deserialize)]
pub struct LessonDetails {
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "cours", deserialize_with = "de_reference_id")]
    pub class: String,
    #[serde(rename = "verrouille")]
    pub locked: bool,
    #[serde(rename = "listeGroupes", deserialize_with = "de_groups")]
    pub groups: Vec<Class>,
    #[serde(rename = "Matiere", deserialize_with = "de_subject")]
    pub subject: Subject,
    #[serde(rename = "CouleurFond")]
    pub color: String,
    #[serde(rename = "listeProfesseurs", deserialize_with = "de_teachers")]
    pub teachers: Vec<Teacher>,
    #[serde(rename = "Date", deserialize_with = "de_datetime")]
    pub start: String,
    #[serde(rename = "DateFin", deserialize_with = "de_datetime")]
    pub end: String,
    #[serde(rename = "dateTAF", deserialize_with = "de_date_some", default)]
    pub homework_date: Option<String>,
    #[serde(rename = "listeContenus", deserialize_with = "de_lesson_details_items")]
    pub items: Vec<LessonDetailsItem>, // + listeElementsProgrammeCDT
}

// i really dont like this name but its hard
// to find one that describes well this struct
#[derive(Debug, Clone, Deserialize)]
pub struct LessonDetailsItem {
    #[serde(rename = "L", default)]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String, // what the lessons refer to
    #[serde(rename = "descriptif", deserialize_with = "de_string")]
    pub description: String,
    #[serde(rename = "categorie", deserialize_with = "de_category")]
    pub category: Category,
    #[serde(rename = "ListePieceJointe", deserialize_with = "de_attachments")]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Agenda {
    #[serde(rename = "ListeEvenements")]
    pub events: Vec<Event>,
    #[serde(rename = "listeFamilles", deserialize_with = "de_event_types")]
    pub event_types: Vec<EventType>,
    // dateDebutPrevisionnel, dateFinPrevisionnel = ???
}

#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    #[serde(rename = "L")]
    pub title: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "DateDebut", deserialize_with = "de_datetime")]
    pub start: String,
    #[serde(rename = "DateFin", deserialize_with = "de_datetime")]
    pub end: String,
    #[serde(rename = "datePublication", deserialize_with = "de_date_some", default)]
    pub date_published: Option<String>,
    #[serde(rename = "Commentaire")]
    pub description: String,
    #[serde(rename = "estConseilClasse")]
    pub is_class_council: bool,
    #[serde(rename = "CouleurCellule")]
    pub color: String,
    #[serde(rename = "estPeriodique")]
    pub is_periodic: bool,
    #[serde(rename = "PiecesJointes", default)]
    pub attachments: Vec<Attachment>,
    // listeFichiers = ???
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventType {
    #[serde(rename = "L")]
    pub title: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "couleur")]
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct School {
    #[serde(rename = "url")] // unnecessary
    pub url: String,
    #[serde(rename = "nomEtab")]
    pub name: String,
    #[serde(rename = "lat", deserialize_with = "de_f64")]
    pub latitude: f64,
    #[serde(rename = "long", deserialize_with = "de_f64")]
    pub longitude: f64,
    #[serde(rename = "cp")]
    pub city_code: String,
}


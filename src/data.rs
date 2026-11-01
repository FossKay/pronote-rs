use crate::helpers::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginMode {
    Password,
    QrCode,
    Token,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QrCode {
    #[serde(rename = "login")]
    pub username: String,
    #[serde(rename = "jeton")]
    pub token: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
}

impl Period {
    pub fn serialize(&self) -> Value {
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
    #[serde(rename = "dateDebut", deserialize_with = "unwrap_v")]
    pub first_day: String,
    #[serde(rename = "dateFin", deserialize_with = "unwrap_v")]
    pub last_day: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Person {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "G")]
    pub group: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaffTeacher {
    #[serde(flatten)]
    pub person: Person,
    #[serde(rename = "estProfesseurPrincipal", default)]
    pub is_primary_teacher: bool,
    #[serde(rename = "matieres", deserialize_with = "unwrap_v")]
    pub subjects: Vec<StaffSubject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewsAuthor {
    #[serde(flatten)]
    pub person: Person,
    #[serde(rename = "avecDiscussion", default)]
    pub with_discussion: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Student {
    #[serde(flatten)]
    pub person: Person,
    #[serde(rename = "classeDEleve")]
    pub class: Class,
    #[serde(rename = "listeOngletsPourPeriodes", deserialize_with = "de_periods")]
    pub periods: Periods,
    #[serde(rename = "listeGroupes", deserialize_with = "unwrap_v")]
    pub groups: Vec<Class>,
}

impl Person {
    pub fn serialize(&self) -> Value {
        json!({
            "G": self.group.unwrap(),
            "L": self.name,
            "N": self.id,
        })
    }
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
        root_url: &str,
        is_solution: bool,
    ) -> String {
        // {{ escapes the "{" character (in format!)
        let json = if !is_solution {
            format!(r#"{{"N":"{}","Actif":true}}"#, self.id)
        } else {
            // decrypted the payload of a solution url
            format!(r#"{{"N":"{}","G":"DevoirCorrige","Actif":true}}"#, self.id)
        };
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaffSubject {
    #[serde(flatten)]
    pub subject: Subject,
    #[serde(rename = "volumeHoraire")]
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GradeSubject {
    #[serde(flatten)]
    pub subject: Subject,
    #[serde(rename = "couleur")]
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GradesSubject {
    #[serde(flatten)]
    pub subject: Subject,
    #[serde(rename = "ordre")]
    pub order: u32,
    #[serde(flatten)]
    pub details: GradesSubjectDetails,
    // theoritically, we would need a custom deserializer so that the
    // type is Option<...> and the fields in the struct aren't default
}

#[derive(Debug, Clone, Deserialize)]
pub struct GradesSubjectDetails {
    #[serde(rename = "couleur", default)]
    pub color: String,
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

#[derive(Debug, Clone, Deserialize)]
pub struct Homework {
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "ordre", default)]
    pub order: u32,
    #[serde(rename = "descriptif", deserialize_with = "unwrap_v")]
    pub description: String,
    #[serde(rename = "TAFFait")]
    pub done: bool,
    #[serde(rename = "niveauDifficulte")]
    pub difficulty: u32,
    #[serde(rename = "duree")]
    pub duration: f32, // minutes
    #[serde(rename = "DonneLe", alias = "donneLe", deserialize_with = "unwrap_v")]
    pub creation_date: String, //NaiveDate,
    #[serde(rename = "PourLe", alias = "pourLe", deserialize_with = "unwrap_v")]
    pub due_date: String, //NaiveDate,
    #[serde(rename = "Matiere", alias = "matiere", deserialize_with = "unwrap_v")]
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
        deserialize_with = "unwrap_v"
    )]
    pub attachments: Vec<Attachment>,
    // TODO: peuRendre
}

#[derive(Default, Debug, Clone)]
pub struct LessonContent {
    pub subject: Subject,
    pub teachers: Vec<String>,
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
    #[serde(rename = "originesCategorie", deserialize_with = "unwrap_v")]
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
    #[serde(rename = "DateDuCours", deserialize_with = "unwrap_v")]
    pub start: String,
    #[serde(rename = "DateDuCoursFin", default)]
    pub end: String,
    #[serde(rename = "CouleurFond")]
    pub color: String,
    #[serde(rename = "ListeContenus", deserialize_with = "de_lesson_content")]
    pub content: LessonContent,
    #[serde(rename = "memo", default)]
    pub note: String,
    #[serde(rename = "cahierDeTextes", deserialize_with = "unwrap_v", default)]
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
    #[serde(rename = "PremierLundi", deserialize_with = "unwrap_v")]
    pub first_day: String,
    #[serde(rename = "DerniereDate", deserialize_with = "unwrap_v")]
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
    #[serde(rename = "listeJoursFeries", deserialize_with = "unwrap_v")]
    pub holidays: Vec<Holiday>,
    #[serde(rename = "NomEtablissement")]
    pub school_name: String,
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
    #[serde(rename = "dateDebut", deserialize_with = "unwrap_v")]
    pub start: String,
    #[serde(rename = "dateFin", deserialize_with = "unwrap_v")]
    pub end: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserParams {
    #[serde(rename = "ressource")]
    pub student: Student,
    #[serde(rename = "listeOnglets", deserialize_with = "de_tabs")]
    pub allowed_tabs: Vec<u64>,
    #[serde(rename = "autorisations")]
    pub autorisations: Permissions,
    #[serde(rename = "infosDroitDeconnexion")]
    pub disconnection: DisconnectionInfos,
    #[serde(skip)]
    pub notifications: Notifications,
    #[serde(skip)]
    pub image: String,
    // TODO: really parse listeOnglets:
    // hierarchical structure like in website -> fetch traductions.js / keep local of tab names
    // (for eventual desktop app)
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Notifications {
    pub global: u32,
    pub discussions: u32,
    pub digital_locker: u32,
    pub informations: u32,
    pub skills_tracking: u32,
    pub media_library_blog: u32,
    pub refresh_message: bool, // polling ig
}

#[derive(Debug, Clone, Deserialize)]
pub struct Permissions {
    #[serde(rename = "AvecDiscussion")]
    pub discussions: bool,
    #[serde(rename = "AvecDiscussionPersonnels")]
    pub student_discussions: bool,
    #[serde(rename = "AvecDiscussionProfesseurs")]
    pub teacher_discussions: bool,
    #[serde(rename = "avecJustificationAbsence")]
    pub justify_absence: bool,
    #[serde(rename = "avecJustificationRetard")]
    pub justify_late: bool,
    #[serde(rename = "avecDroitDeconnexionMessagerie")]
    pub discussions_deconnection: bool,
    #[serde(rename = "tailleMaxDocJointEtablissement")]
    pub teacher_attachment_max_size: f32, // bytes, eg. 5242880 = 5MiB
    #[serde(rename = "tailleMaxRenduTafEleve")]
    pub student_attachment_max_size: f32,
    #[serde(rename = "tailleTravailAFaire")]
    pub homework_size: u32, // not max apparently
    #[serde(rename = "tailleCirconstance")]
    pub circumstance_size: u32,
    #[serde(rename = "tailleCommentaire")]
    pub note_size: u32,
    #[serde(rename = "consulterDonneesAdministrativesAutresEleves")]
    pub view_others_administrative_data: bool, // true???
    #[serde(rename = "compte")]
    pub account: AccountPermissions,
    #[serde(rename = "autoriserImpressionBulletinReleveBrevet", default)]
    pub can_print_brevet_report: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountPermissions {
    #[serde(rename = "avecSaisieMotDePasse", default)]
    pub password: bool,
    #[serde(rename = "avecInformationsPersonnelles", default)]
    pub personal_informations: bool,
    #[serde(rename = "avecSaisieInfosPersoCoordonnees", default)]
    pub personal_informations_entry: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisconnectionInfos {
    #[serde(rename = "notes")]
    pub grades: GradesDisconnection,
    #[serde(rename = "taf")]
    pub homework: HomeworkDisconnection,
    #[serde(rename = "messagerie")]
    pub deconnection: ChatsDisconnection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GradesDisconnection {
    #[serde(rename = "heurePublicationNote")]
    pub publication_time: u8,
    #[serde(rename = "nombreJoursDecalagePublicationNote")]
    pub publication_delay: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeworkDisconnection {
    #[serde(rename = "heureDebutDesactivationPublication")]
    pub start_time: u8,
    #[serde(rename = "heureFinDesactivationPublication")]
    pub end_time: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatsDisconnection {
    #[serde(rename = "listeJoursOuvresDeconnexion")]
    pub restricted_days: Vec<String>,
    #[serde(rename = "listeJoursNonOuvresDeconnexion")]
    pub blocked_days: Vec<String>, // non ouvrés???
    #[serde(rename = "avecJourFerieDeconnexion")]
    pub holidays_blocked: bool,
    #[serde(rename = "heureAvantDeconnexion")]
    pub start_time: u8,
    #[serde(rename = "heureApresDeconnexion")]
    pub end_time: u8,
}

#[derive(Default, Debug, Clone)]
pub struct Periods {
    pub periods: Vec<Period>,
    pub default: Period,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Home {
    #[serde(rename = "estDansUnePeriodeDeDeconnexion")]
    pub is_disconnection: bool,
    #[serde(rename = "infosDroitDeconnexion")]
    pub disconnection: DisconnectionInfos,
    #[serde(rename = "dateSelectionnee", deserialize_with = "unwrap_v")]
    pub date: String,
    #[serde(rename = "prochaineDate", deserialize_with = "unwrap_v", default)]
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
    #[serde(rename = "actualites", deserialize_with = "de_home_news")]
    pub news: Vec<NewsItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeAttachments {
    #[serde(rename = "listeRessources", deserialize_with = "unwrap_v")]
    pub attachments: Vec<HomeAttachment>,
    #[serde(rename = "listeMatieres", deserialize_with = "unwrap_v")]
    pub subjects: Vec<Subject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeAttachment {
    #[serde(rename = "ressource", deserialize_with = "unwrap_v")]
    pub attachment: Attachment,
    #[serde(rename = "date", deserialize_with = "unwrap_v")]
    pub date: String,
    #[serde(rename = "matiere", deserialize_with = "de_reference_id")]
    pub subject_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeGrades {
    pub grades: Vec<Grade>,
    pub period: Option<Period>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Exam {
    #[serde(rename = "L", default)]
    pub title: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "matiere", deserialize_with = "unwrap_v")]
    pub subject: Subject,
    #[serde(rename = "couleur")]
    pub color: String,
    #[serde(rename = "dateDebut", deserialize_with = "unwrap_v")]
    pub start: String,
    #[serde(rename = "dateFin", deserialize_with = "unwrap_v")]
    pub end: String,
    #[serde(rename = "listeSalles", deserialize_with = "unwrap_v")]
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
    #[serde(rename = "listeServices", deserialize_with = "unwrap_v")]
    pub subjects: Vec<GradesSubject>,
    #[serde(rename = "listeDevoirs", deserialize_with = "unwrap_v")]
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
    #[serde(rename = "date", deserialize_with = "unwrap_v")]
    pub date: String,
    #[serde(rename = "service", deserialize_with = "unwrap_v")]
    pub subject: GradeSubject,
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
    #[serde(rename = "periode", deserialize_with = "unwrap_v")]
    pub period: Period,
    #[serde(rename = "elmCorrige", deserialize_with = "unwrap_v", default)]
    pub solution: Option<Attachment>, // there is also libelleCorrige
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
    Error,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Resources {
    #[serde(rename = "ListeCahierDeTextes", deserialize_with = "unwrap_v")]
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
    #[serde(rename = "description", deserialize_with = "unwrap_v")]
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
    #[serde(rename = "listeGroupes", deserialize_with = "unwrap_v")]
    pub groups: Vec<Class>,
    #[serde(rename = "Matiere", deserialize_with = "unwrap_v")]
    pub subject: Subject,
    #[serde(rename = "CouleurFond")]
    pub color: String,
    #[serde(rename = "listeProfesseurs", deserialize_with = "unwrap_v")]
    pub teachers: Vec<Person>,
    #[serde(rename = "Date", deserialize_with = "unwrap_v")]
    pub start: String,
    #[serde(rename = "DateFin", deserialize_with = "unwrap_v")]
    pub end: String,
    #[serde(rename = "dateTAF", deserialize_with = "unwrap_v", default)]
    pub homework_date: Option<String>,
    #[serde(rename = "listeContenus", deserialize_with = "unwrap_v")]
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
    #[serde(rename = "descriptif", deserialize_with = "unwrap_v")]
    pub description: String,
    #[serde(rename = "categorie", deserialize_with = "unwrap_v")]
    pub category: Category,
    #[serde(rename = "ListePieceJointe", deserialize_with = "unwrap_v")]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Agenda {
    #[serde(rename = "ListeEvenements")]
    pub events: Vec<Event>,
    #[serde(rename = "listeFamilles", deserialize_with = "unwrap_v")]
    pub event_types: Vec<EventType>,
    // dateDebutPrevisionnel, dateFinPrevisionnel = ???
}

#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    #[serde(rename = "L")]
    pub title: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "DateDebut", deserialize_with = "unwrap_v")]
    pub start: String, // datetime OR date
    #[serde(rename = "DateFin", deserialize_with = "unwrap_v")]
    pub end: String, // datetime OR date
    #[serde(rename = "datePublication", deserialize_with = "unwrap_v", default)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct News {
    // infos + polls
    #[serde(rename = "listeNatures", deserialize_with = "unwrap_v")]
    pub categories: Vec<NewsCategory>,
    #[serde(rename = "listeModesAff", deserialize_with = "de_news")]
    pub news: Vec<NewsItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewsCategory {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "estDefaut", default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewsDetails {
    #[serde(rename = "avecPJ")]
    pub with_attachment: bool,
    #[serde(rename = "aToutRepondu")]
    pub replied_to_everything: bool,
    #[serde(rename = "estRepondant", default)]
    pub is_replier: bool,
    #[serde(rename = "strRepondant", default)]
    pub replier_name: Option<String>,
    #[serde(rename = "reponduLe", deserialize_with = "unwrap_v", default)]
    pub reply_date: Option<String>,
    #[serde(rename = "avecCommandeVisuResultat")]
    pub show_results: bool, // ??? ig
}

// serialize for get content request:
// - nature, elmauteur, public no longer {"_T": ..., V": ...}
// - new fields:
// {
//   "Actif": true,
//   "listeCategories": [],
//   "DateDebut": ... same as dateDebut ...,
//   "estSelectionne": true,
//   "listeQuestions": []
// }

// unnecessary:
// - L
// - reponseAnonyme
// - estInformation
// - estSondage
// - lue
// - nature
// - dateDebut
// - dateFin
// - estProlonge
// - dateCreation
// - auteur
// - estAuteur
// - elmauteur
// - prenom
// - estPublic
// - estPartage
// - informationListeContenu
// - DateDebut
// - Actif
// - estSelectionne
// - listeQuestions

// keep:
// N, public, genrePublic, listeCategories

#[derive(Debug, Clone, Deserialize)]
pub struct NewsItem {
    #[serde(rename = "L")]
    pub title: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "reponseAnonyme")]
    pub anonymous_response: bool,
    #[serde(rename = "estInformation")]
    pub is_information: bool,
    #[serde(rename = "estSondage")]
    pub is_poll: bool,
    #[serde(rename = "nature", deserialize_with = "unwrap_v")]
    pub category: NewsCategory,
    #[serde(rename = "lue")]
    pub viewed: bool,
    #[serde(rename = "dateDebut", deserialize_with = "unwrap_v")]
    pub start: String,
    #[serde(rename = "dateFin", deserialize_with = "unwrap_v")]
    pub end: String,
    #[serde(rename = "estProlonge")]
    pub is_extended: bool,
    #[serde(rename = "dateCreation", deserialize_with = "unwrap_v")]
    pub creation_date: String,
    #[serde(rename = "auteur")]
    pub author_name: String,
    #[serde(rename = "estAuteur")]
    pub is_author: bool, // should be false for students ig
    #[serde(rename = "elmauteur", deserialize_with = "unwrap_v")]
    pub author: NewsAuthor,
    #[serde(rename = "prenom")]
    pub student_name: String,
    #[serde(rename = "public", deserialize_with = "unwrap_v")]
    pub student: Person,
    #[serde(rename = "estPublic", default)]
    pub is_public: bool,
    #[serde(rename = "estPartage", default)]
    pub is_shared: bool,
    #[serde(rename = "genrePublic")]
    pub receiver_group: u32, // refer to Person struct
    #[serde(rename = "informationListeContenu", default)]
    pub details: Option<NewsDetails>,
}

impl NewsItem {
    pub fn serialize(&self) -> Value {
        json!({
            "N": self.id,
            "public": self.student.clone().serialize(),
            "genrePublic": self.receiver_group,
            "listeCategories": [],
        })
    }
}

// when fetching news item content, it is named "question", probably for polls
#[derive(Debug, Clone, Deserialize)]
pub struct Question {
    #[serde(rename = "L")]
    pub name: String, // Question 1???
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "P")]
    pub p: u32, // idk maybe place
    #[serde(rename = "rang")]
    pub rank: u32,
    #[serde(rename = "genreReponse")]
    pub response_type: u32,
    #[serde(rename = "titre")]
    pub title: String,
    #[serde(rename = "texte", deserialize_with = "unwrap_v")]
    pub content: String,
    #[serde(rename = "tailleReponse")]
    pub response_size: u32,
    #[serde(rename = "avecMaximum")]
    pub is_maximum: bool, // maybe refers to response_size?
    #[serde(rename = "nombreReponsesMax")]
    pub max_responses: u32,
    #[serde(rename = "listePiecesJointes", deserialize_with = "unwrap_v")]
    pub attachments: Vec<Attachment>,
    // listeChoix: {"_T": 24, "V": []}
    #[serde(rename = "reponse", deserialize_with = "unwrap_v")]
    pub response: Response,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "avecReponse")]
    pub with_response: bool,
    #[serde(rename = "estReponseAttendue")]
    pub response_needed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscussionLabel {
    #[serde(rename = "L")]
    pub name: String,
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "G")]
    pub numeric_id: u32,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Possession {
    #[serde(rename = "N")]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Discussion {
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "estUneDiscussion", default)]
    pub is_discussion: bool, // because the discussions json is a mess :(
    #[serde(rename = "nombreMessages", default)]
    pub message_count: u32,
    #[serde(rename = "objet", default)]
    pub title: String,
    #[serde(
        rename = "listePossessionsMessages",
        deserialize_with = "unwrap_v",
        default
    )]
    pub possessions: Vec<Possession>,
    #[serde(
        rename = "messagePourParticipants",
        deserialize_with = "de_reference_id",
        default
    )]
    pub participants_message_id: String,
    #[serde(rename = "nbPublic", default)]
    pub participants_count: u32,
    #[serde(rename = "initiateur", default)]
    pub author: String,
    #[serde(
        rename = "dernierPossessionMessage",
        deserialize_with = "unwrap_v",
        default
    )]
    pub last_possession: Possession,
    #[serde(
        rename = "messageFenetre",
        deserialize_with = "de_reference_id",
        default
    )]
    pub message_window_id: String,
    #[serde(rename = "lu", default)]
    pub viewed: bool,
    #[serde(rename = "documentsJoints", default)]
    pub attachments: String, // separated by \n
    #[serde(rename = "libelleDate", default)]
    pub pretty_date: String,
    #[serde(rename = "estNonPossede")]
    pub is_not_possessed: bool, // ?????
    #[serde(rename = "indicePere", default)]
    pub parent_id: u32, // find parent_id: 0 => id: "0"
    #[serde(rename = "profondeur")]
    pub depth: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Discussions {
    #[serde(rename = "listeEtiquettes", deserialize_with = "unwrap_v")]
    pub labels: Vec<DiscussionLabel>,
    #[serde(rename = "listeMessagerie", deserialize_with = "unwrap_v")]
    pub discussions: Vec<Discussion>,
    #[serde(rename = "messagerieDesactivee")]
    pub is_disconnection: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Button {
    #[serde(rename = "L")]
    pub id: String,
    #[serde(rename = "G")]
    pub group: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Messages {
    #[serde(rename = "listeMessages", deserialize_with = "unwrap_v")]
    pub messages: Vec<Message>,
    #[serde(rename = "messagePourReponse", deserialize_with = "de_reference_id")]
    pub reply_message_id: String, // should also store nbDestinataires but it's
    // like the receivers_count of the last message?
    #[serde(rename = "listeBoutons", deserialize_with = "unwrap_v")]
    pub buttons: Vec<Button>,
    #[serde(rename = "nbPossessionsMessage")]
    pub possessions_count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[serde(rename = "N")]
    pub id: String,
    #[serde(rename = "messageSource", deserialize_with = "de_reference_id")]
    pub discussion_id: String,
    #[serde(rename = "possessionMessage", deserialize_with = "unwrap_v")]
    pub possession: Possession,
    #[serde(rename = "estNonPossede")]
    pub is_not_possessed: bool, // ???
    #[serde(rename = "contenu", deserialize_with = "unwrap_v")]
    pub content: String,
    #[serde(rename = "estHTML")]
    pub is_html: bool,
    #[serde(rename = "libelleDate")]
    pub pretty_date: String,
    #[serde(rename = "date", deserialize_with = "unwrap_v")]
    pub date: String,
    #[serde(rename = "estUnAparte")]
    pub is_aparte: bool, // ???
    #[serde(
        rename = "listeDocumentsJoints",
        deserialize_with = "unwrap_v",
        default
    )] // maybe no default?
    pub attachments: Vec<Attachment>,
    #[serde(rename = "emetteur")]
    pub is_sender: bool,
    #[serde(rename = "public_gauche")]
    pub sender_name: String,
    #[serde(rename = "hint_gauche")]
    pub sender_role: String, // eg. Professeur
    #[serde(rename = "nbPublic")]
    pub receivers_count: u32,
}

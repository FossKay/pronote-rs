use crate::data::*;
use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde_json::{Value, from_value};

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
                name: name.to_owned(),
                ..Default::default()
            },
            teachers: Vec::new(),
            classroom: Vec::new(),
            group: None,
        },
        note: "".into(),
        lesson_details_link: None,
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

pub fn parse_notifications(res: &Value) -> Notifications {
    let mut n = Notifications {
        global: res["notifications"]["compteurCentraleNotif"].as_u64().unwrap() as u32,
        discussions: 0,
        digital_locker: 0,
        informations: 0,
        skills_tracking: 0,
        media_library_blog: 0,
        refresh_message: res["actualisationMessage"].as_bool().unwrap(),
    };
    for i in res["notificationsCommunication"].as_array().unwrap() {
        let number = i["nb"].as_u64().unwrap() as u32;
        match i["onglet"].as_u64().unwrap() {
            131 => n.discussions = number,
            148 => n.digital_locker = number,
            8 => n.informations = number,
            275 => n.skills_tracking = number,
            253 => n.media_library_blog = number,
            _ => unreachable!(),
        }
    }

    n
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

// unwraps {"_T": XX, "V": ...}
macro_rules! deser {
    ($type:ty, $name:ident) => {
        paste::paste! {
            pub fn [<de_ $name>]<'de, D>(deserializer: D) -> Result<$type, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Value::deserialize(deserializer)?["V"].clone();
                let res: $type = from_value(value).unwrap();
                Ok(res)
            }
        }
    };
}

deser!(String, string);
deser!(String, date); //parse_date(...)
deser!(String, datetime); //parse_datetime(...)
deser!(Option<String>, date_some);
deser!(Option<String>, datetime_some);
deser!(Vec<Grade>, grades);
deser!(Vec<LessonDetails>, lesson_details);
deser!(Person, person);
deser!(Vec<Person>, persons);
deser!(Period, period);
deser!(Vec<Class>, groups);
deser!(Vec<LessonDetailsItem>, lesson_details_items);
deser!(Category, category);
deser!(Vec<Category>, categories);
deser!(Option<LessonDetailsLink>, lesson_details_link);
deser!(Attachment, attachment);
deser!(Option<Attachment>, attachment_some);
deser!(Vec<Attachment>, attachments);
deser!(Vec<HomeAttachment>, home_attachments);
deser!(Subject, subject);
deser!(Vec<Subject>, subjects);
deser!(Vec<Holiday>, holidays);
deser!(Vec<EventType>, event_types);
deser!(Vec<Classroom>, classrooms);
deser!(NewsCategory, news_category);
deser!(Vec<NewsCategory>, news_categories);
deser!(Possession, possession);
deser!(Vec<Possession>, possessions);
deser!(Vec<Discussion>, discussions);
deser!(Vec<DiscussionLabel>, discussion_labels);
deser!(Vec<Message>, messages);
deser!(Vec<Button>, buttons);

macro_rules! deser2 {
    ($type:ty, $name:ident, $body:expr) => {
        paste::paste! {
            pub fn [<de_ $name>]<'de, D>(
                deserializer: D,
            ) -> Result<$type, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = Value::deserialize(deserializer)?.clone();
                ($body)(value)
            }
        }
    };
}

deser2!(HomeGrades, home_grades, |value: Value| {
    let grades: Vec<Grade> = from_value(value["listeDevoirs"]["V"].clone()).unwrap();
    let period: Period = from_value(value["page"]["periode"]["V"].clone()).unwrap();

    Ok(HomeGrades { grades, period })
});
deser2!(Vec<DigitalTextbook>, digital_textbooks, |value: Value| {
    let textbooks: Vec<DigitalTextbook> =
        from_value(value["V"]["listeRessources"]["V"].clone()).unwrap();

    Ok(textbooks)
});
deser2!(Vec<DigitalTextbook>, digital_textbooks2, |value: Value| {
    let textbooks: Vec<DigitalTextbook> =
        from_value(value["listeRessources"]["V"].clone()).unwrap();

    Ok(textbooks)
});
deser2!(Vec<Homework>, homework, |value: Value| {
    let mut res: Vec<Homework> = from_value(value["listeTAF"]["V"].clone()).unwrap();
    res.sort_by_key(|x| x.order);

    Ok(res)
});
deser2!(Vec<Event>, home_agenda, |value: Value| {
    let events: Vec<Event> = from_value(value["listeEvenements"].clone()).unwrap();

    Ok(events)
});
deser2!(Vec<Exam>, exams, |value: Value| {
    let events: Vec<Exam> = from_value(value["listeDS"]["V"].clone()).unwrap();

    Ok(events)
});
deser2!(Vec<NewsItem>, home_news, |value: Value| {
    let events: Vec<NewsItem> =
        from_value(value["listeModesAff"][0]["listeActualites"]["V"].clone()).unwrap();

    Ok(events)
});
deser2!(Vec<NewsItem>, news, |value: Value| {
    let events: Vec<NewsItem> = from_value(value[0]["listeActualites"]["V"].clone()).unwrap();

    Ok(events)
});
deser2!(Vec<EndTime>, end_times, |value: Value| {
    let mut res = Vec::new();
    for v in value["V"].clone().as_array().unwrap() {
        res.push(EndTime {
            time: v["L"].as_str().unwrap().into(),
            show: v.get("A").and_then(Value::as_bool).unwrap_or(true),
        })
    }

    Ok(res)
});
deser2!(Vec<u64>, tabs, |value: Value| {
    Ok(extract_onglets(value))
});
deser2!(Periods, periods, |value: Value| {
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
        .map(|json| from_value::<Period>(json).unwrap())
        .collect();
    let default_period: Period = from_value(entries["periodeParDefaut"]["V"].clone()).unwrap();

    Ok(Periods {
        periods: periods,
        default: default_period,
    })
});
deser2!(LessonContent, lesson_content, |value: Value| {
    let values: Vec<Value> = value["V"].as_array().cloned().unwrap_or_default();

    let mut subject: Option<Subject> = None;
    let mut teachers: Vec<String> = Vec::new();
    let mut classroom: Vec<Classroom> = Vec::new();
    let mut group = None;

    for value in values {
        match value["G"].as_u64() {
            Some(16) => {
                subject = Some(from_value(value).map_err(serde::de::Error::custom)?);
            }
            Some(3) => {
                if let Some(t) = value["L"].as_str() {
                    teachers.push(t.to_owned());
                }
            }
            Some(2) => {
                group = value["L"].as_str().map(String::from);
            }
            Some(17) => {
                classroom.push(from_value(value).map_err(serde::de::Error::custom)?);
            }
            _ => {}
        }
    }

    Ok(LessonContent {
        subject: subject.unwrap(),
        teachers,
        classroom,
        group,
    })
});
deser2!(GradeValue, grade, |value: Value| {
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
});
deser2!(u32, bareme, |value: Value| {
    Ok(value["V"].as_str().unwrap().parse().unwrap())
});
deser2!(f64, f64, |value: Value| {
    Ok(value.as_str().unwrap().parse().unwrap())
});
deser2!(Option<String>, color, |value: Value| {
    Ok(value["ligne"].as_str().map(String::from))
});
deser2!(String, sticky_note, |value: Value| {
    Ok(value["libelle"]
        .as_str()
        .map(String::from)
        .unwrap_or_default())
});
deser2!(String, reference_id, |value: Value| {
    Ok(value["V"]["N"].as_str().unwrap().into())
});

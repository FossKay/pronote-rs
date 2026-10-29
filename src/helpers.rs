use crate::data::*;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_json::{Value, from_value};

pub fn pretty_end_time(end_time: u32, l: &Lesson, p: &Params) -> String {
    format!(
        "{} {}:00",
        l.start.split_once(' ').unwrap().0,
        p.end_times[(end_time - 1) as usize].time.replace('h', ":")
        // use char when matching single character
    )
}

fn gap(id: &str, name: &str, index: u32, duration: u32, l: &Lesson, p: &Params) -> Lesson {
    Lesson {
        id: id.into(),
        order: 1000,
        status: String::new(),
        cancelled: false,
        index,
        duration,
        start: pretty_end_time(index, l, p),
        end: pretty_end_time(index + duration, l, p),
        color: String::new(),
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

pub fn parse_date(date: &str) -> Result<NaiveDate, crate::Error> {
    NaiveDate::parse_from_str(date, "%d/%m/%Y").map_err(|_| crate::Error::InvalidResponse)
}

pub fn parse_datetime(date: &str) -> Result<NaiveDateTime, crate::Error> {
    NaiveDateTime::parse_from_str(date, "%d/%m/%Y %H:%M:%S")
        .map_err(|_| crate::Error::InvalidResponse)
}

fn extract_tabs(v: &Value, res: &mut Vec<u64>) {
    res.push(v["G"].as_u64().unwrap());
    if let Some(more_tabs) = v.get("Onglet") {
        for tab in more_tabs.as_array().unwrap() {
            extract_tabs(tab, res);
        }
    }
}

pub fn parse_attachments(res: &Value) -> Vec<String> {
    if let Some(files) = res.get("fichiers") {
        let mut a = Vec::new();
        for r in files.as_array().unwrap() {
            a.push(r.as_str().unwrap().replace("\r\n", ""))
        }
        return a;
    }
    return Vec::new();
}

pub fn unwrap_v<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let mut value = Value::deserialize(deserializer)?;
    //println!("{value:#?}");
    let inner = value["V"].take();

    Ok(serde_json::from_value(inner).unwrap())
}

macro_rules! deser {
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

deser!(HomeGrades, home_grades, |value: Value| {
    let grades: Vec<Grade> = from_value(value["listeDevoirs"]["V"].clone()).unwrap();
    let period: Period = from_value(value["page"]["periode"]["V"].clone()).unwrap();

    Ok(HomeGrades { grades, period })
});
deser!(Vec<DigitalTextbook>, digital_textbooks, |value: Value| {
    let textbooks: Vec<DigitalTextbook> =
        from_value(value["V"]["listeRessources"]["V"].clone()).unwrap();

    Ok(textbooks)
});
deser!(Vec<DigitalTextbook>, digital_textbooks2, |value: Value| {
    let textbooks: Vec<DigitalTextbook> =
        from_value(value["listeRessources"]["V"].clone()).unwrap();

    Ok(textbooks)
});
deser!(Vec<Homework>, homework, |value: Value| {
    let mut res: Vec<Homework> = from_value(value["listeTAF"]["V"].clone()).unwrap();
    res.sort_by_key(|x| x.order);

    Ok(res)
});
deser!(Vec<Event>, home_agenda, |value: Value| {
    let events: Vec<Event> = from_value(value["listeEvenements"].clone()).unwrap();

    Ok(events)
});
deser!(Vec<Exam>, exams, |value: Value| {
    let events: Vec<Exam> = from_value(value["listeDS"]["V"].clone()).unwrap();

    Ok(events)
});
deser!(Vec<NewsItem>, home_news, |value: Value| {
    let events: Vec<NewsItem> =
        from_value(value["listeModesAff"][0]["listeActualites"]["V"].clone()).unwrap();

    Ok(events)
});
deser!(Vec<NewsItem>, news, |value: Value| {
    let events: Vec<NewsItem> = from_value(value[0]["listeActualites"]["V"].clone()).unwrap();

    Ok(events)
});
deser!(Vec<EndTime>, end_times, |value: Value| {
    let mut res = Vec::new();
    for v in value["V"].clone().as_array().unwrap() {
        res.push(EndTime {
            time: v["L"].as_str().unwrap().into(),
            show: v.get("A").and_then(Value::as_bool).unwrap_or(true),
        })
    }

    Ok(res)
});
deser!(Vec<u64>, tabs, |value: Value| {
    let mut tabs = Vec::new();
    for i in value.as_array().unwrap() {
        extract_tabs(i, &mut tabs);
    }
    Ok(tabs)
});
deser!(Periods, periods, |value: Value| {
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
deser!(LessonContent, lesson_content, |value: Value| {
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
deser!(GradeValue, grade, |value: Value| {
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
        "|-1" => GradeValue::Error,
        _ => s.replace(',', ".").parse().map(GradeValue::Value).unwrap(),
    })
});
deser!(u32, bareme, |value: Value| {
    Ok(value["V"].as_str().unwrap().parse().unwrap())
});
deser!(f64, f64, |value: Value| {
    Ok(value.as_str().unwrap().parse().unwrap())
});
deser!(Option<String>, color, |value: Value| {
    Ok(value["ligne"].as_str().map(String::from))
});
deser!(String, sticky_note, |value: Value| {
    Ok(value["libelle"]
        .as_str()
        .map(String::from)
        .unwrap_or_default())
});
deser!(String, reference_id, |value: Value| {
    Ok(value["V"]["N"].as_str().unwrap().into())
});

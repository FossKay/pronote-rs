use pronote_rs::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    /* let mut c = Client::new(
        "https://demo.index-education.net/pronote/eleve.html".into(),
        "demonstration".into(),
        "pronotevs".into(),
    ).await?;

    println!("{:?}", c.user_parameters.unwrap()); */

    let params = Client::mock_params().await?;

    println!("Timetable: {:?}", Client::mock_timetable(&params).await?);
    println!("Home: {:?}", Client::mock_home(&params).await?);
    println!("Homework: {:?}", Client::mock_homework().await?);
    println!(
        "Grades: {:?}",
        Client::mock_grades(Period {
            id: "105#x8KmCqPkfy0upIPQ4OlBec-oQBVtwKvAVIbivyTMd7U".into(),
            name: "Semestre 1".into(),
        })
        .await?
    );

    Ok(())
}

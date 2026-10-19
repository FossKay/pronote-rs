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
            id: "105#NV-t6VP4ljPbJwpOw5CL-PQPtwZ5Mq_noj0oWMjgDmg".into(),
            name: "Trimestre 1".into(),
        })
        .await?
    );
    println!("Lesson details & documents: {:?}", Client::mock_documents().await?);

    Ok(())
}

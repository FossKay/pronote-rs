use pronote_rs::*;

// cargo test, shouldn't panic
#[tokio::test]
async fn mock_test() -> Result<(), Error> {
    let params = Client::mock_params().await?;
    let user_params = Client::mock_user_params().await?;

    Client::mock_homework().await?;
    Client::mock_documents().await?;
    Client::mock_timetable(&params).await?;
    Client::mock_home(&params).await?;
    Client::mock_agenda().await?;
    Client::mock_news().await?;
    Client::mock_news_item_content().await?;
    Client::mock_discussions().await?;
    Client::mock_messages().await?;
    Client::mock_geolocation().await?;

    for period in &user_params.student.periods.periods {
        Client::mock_grades(period).await?;
    }

    Ok(())
}

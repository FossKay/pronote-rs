use std::io::Write;

use pronote_rs::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut line = String::new();

    print!("Login method [MOCK/DEMO/REAL]: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut line).unwrap();

    let mode = line.trim().to_uppercase();

    let mut c = match mode.to_lowercase().as_str() {
        "m" | "mock" => None,
        "d" | "demo" => Some(
            Client::password_login(
                "https://demo.index-education.net/pronote".into(),
                "demonstration".into(),
                "pronotevs".into(),
            )
            .await?,
        ),
        "r" | "real" => match read("Login mode [QR/TOKEN/PWD]: ").to_lowercase().as_str() {
            "q" | "qr" => {
                let qr = read("Qr code data: ");
                let pin = read("Pin code: ");
                let uuid = &Client::rand_uuid();

                let c = Client::qr_login(&qr, &pin, uuid).await?;

                println!("Credentials to save:");
                println!("Root url: {}", c.communication.root_url);
                println!("Nav identifier: {}", c.nav_identifier);
                println!("Username: {}", c.username);
                println!("Next token: {}", c.next_token);
                println!("Uuid: {}", c.uuid);

                Some(c)
            }
            "t" | "token" => {
                let root = read("Root url: ");
                let nav_identifier = read("Nav identifier: ");
                let user = read("Username: ");
                let token = read("Token: ");
                let uuid = read("Uuid: ");

                let c = Client::token_login(&root, &user, &token, &uuid, &nav_identifier).await?;

                println!("Credentials to save:");
                println!("Root url: {}", c.communication.root_url);
                println!("Nav identifier: {}", c.nav_identifier);
                println!("Username: {}", c.username);
                println!("Next token: {}", c.next_token);
                println!("Uuid: {}", c.uuid);

                Some(c)
            }
            "p" | "pwd" => {
                let root = read("Root url: ");
                let user = read("Username: ");
                let pwd = read("Password: ");

                Some(Client::password_login(&root, &user, &pwd).await?)
            }
            _ => {
                eprintln!("Invalid login mode");
                return Ok(());
            }
        },
        _ => {
            eprintln!("Invalid login method");
            return Ok(());
        }
    };

    loop {
        let command = read("[pronote-rs]$ ");
        let args: Vec<&str> = command.split_whitespace().collect();

        if args.is_empty() {
            continue;
        }

        match args[0] {
            "exit" => break,
            "help" => {
                println!("Available commands:");
                println!(
                    "exit, help, clear, ls, params, userparams, homework, done, documents, home, grades, agenda, news, newsitem, discussions, messages, geolocation"
                );
            }
            "clear" => {
                print!("\x1B[2J\x1B[H");
                std::io::stdout().flush().unwrap();
            }
            "ls" => {
                let periods = match &c {
                    Some(client) => client.user_parameters.clone().unwrap().student.periods,
                    None => Client::mock_user_params().await?.student.periods,
                };
                println!("{periods:#?}");
            }
            "params" => {
                let params = match &c {
                    Some(client) => client.parameters.clone(),
                    None => Client::mock_params().await?,
                };
                println!("{params:#?}");
            }
            "userparams" => {
                let userparams = match &c {
                    Some(client) => client.user_parameters.clone().unwrap(),
                    None => Client::mock_user_params().await?,
                };
                println!("{userparams:#?}");
            }
            "homework" if c.is_none() || args.len() == 3 => {
                let result = match &mut c {
                    None => Client::mock_homework().await?,
                    Some(client) => {
                        let first_week: u8 = args[1].parse().unwrap();
                        let last_week: u8 = args[2].parse().unwrap();
                        client.homework(first_week, last_week).await?
                    }
                };
                println!("{result:#?}");
            }
            "homework" => {
                println!("Usage: homework <first_week:u8> <last_week:u8>");
            }
            "done" if c.is_none() => {
                println!("Cant set homework done in mock env");
            }
            "done" if args.len() == 3 => {
                let done = args[1].to_lowercase() == "y";
                match &mut c {
                    Some(client) => client.set_done(args[2], done).await?,
                    None => println!("Cant set homework done in mock env"),
                }
            }
            "done" => {
                println!("Usage: done [Y/n] <homework_id:String>");
            }
            "documents" if c.is_none() || args.len() == 3 => {
                let result = match &mut c {
                    None => Client::mock_documents().await?,
                    Some(client) => {
                        let first_week: u8 = args[1].parse().unwrap();
                        let last_week: u8 = args[2].parse().unwrap();
                        client.documents(first_week, last_week).await?
                    }
                };
                println!("{result:#?}");
            }
            "documents" => {
                println!("Usage: documents <first_week:u8> <last_week:u8>");
            }
            "timetable" if c.is_none() || args.len() == 2 => {
                let result = match &mut c {
                    None => {
                        let params = Client::mock_params().await?;
                        Client::mock_timetable(&params).await?
                    }
                    Some(client) => {
                        let week: u8 = args[1].parse().unwrap();
                        client.timetable(week).await?
                    }
                };
                println!("{result:#?}");
            }
            "timetable" => {
                println!("Usage: timetable <week:u8>");
            }
            "home" if c.is_none() || args.len() == 2 => {
                let result = match &mut c {
                    None => {
                        let params = Client::mock_params().await?;
                        Client::mock_home(&params).await?
                    }
                    Some(client) => client.home(args[1].into()).await?,
                };
                println!("{result:#?}");
            }
            "home" => {
                println!("Usage: home <date:String (dd/mm/yyyy)>");
            }
            "grades" if args.len() == 2 => {
                let periods = match &c {
                    Some(client) => client.user_parameters.clone().unwrap().student.periods,
                    None => Client::mock_user_params().await?.student.periods,
                };

                let period = match periods
                    .periods
                    .iter()
                    .find(|x| x.name == args[1].replace("_", " "))
                {
                    None => {
                        println!("No such period");
                        println!("Use ls to view the periods");
                        println!("(Tip: use underscores instead of spaces)");
                        continue;
                    }
                    Some(period) => period,
                }
                .clone();

                let result = match &mut c {
                    None => Client::mock_grades(&period).await?,
                    Some(client) => client.grades(&period).await?,
                };
                println!("{result:#?}");
            }
            "grades" => {
                println!("Usage: grades <period_name:String>");
            }
            "agenda" => {
                let result = match &mut c {
                    Some(client) => client.agenda().await?,
                    None => Client::mock_agenda().await?,
                };
                println!("{result:#?}");
            }
            "news" => {
                let result = match &mut c {
                    Some(client) => client.news().await?,
                    None => Client::mock_news().await?,
                };
                println!("{result:#?}");
            }
            "newsitem" if c.is_none() || args.len() > 1 => {
                let result = match &mut c {
                    None => Client::mock_news_item_content().await?,
                    Some(client) => client.news_item_content(args[1]).await?,
                };
                println!("{result:#?}");
            }
            "newsitem" => {
                println!("Usage: newsitem <news_item_id:String>");
            }
            "discussions" => {
                let result = match &mut c {
                    Some(client) => client.discussions().await?,
                    None => Client::mock_discussions().await?,
                };
                println!("{result:#?}");
            }
            "messages" if c.is_none() || args.len() >= 2 => {
                let result = match &mut c {
                    None => Client::mock_messages().await?,
                    Some(client) => {
                        let possessions: Vec<Possession> = args[1..]
                            .iter()
                            .map(|&x| Possession { id: x.to_owned() })
                            .collect();

                        client.messages(&possessions).await?
                    }
                };
                println!("{result:#?}");
            }
            "messages" => {
                println!("Usage: messages <possession:String> (<possession2:String> ...)");
            }
            "geolocation" if c.is_none() || args.len() == 3 => {
                let result = match &mut c {
                    None => Client::mock_geolocation().await?,
                    Some(_) => {
                        let lat: f64 = args[1].parse().unwrap();
                        let long: f64 = args[2].parse().unwrap();
                        Client::geolocation(lat, long).await?
                    }
                };
                println!("{result:#?}");
            }
            "geolocation" => {
                println!("Usage: geolocation <latitude:f64> <longitude:f64>");
            }
            _ => println!("Unknown command: {}", args[0]),
        }
    }
    Ok(())
}

fn read(prompt: &str) -> String {
    print!("{prompt}");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}

# pronote-rs

> This project is not affiliated with PRONOTE nor Index Education

[Lire ce README en français](README_FR.md)

An unofficial rust crate to interact with the french PRONOTE school website.
Only student profiles are supported for now.

## Feature list
- Authentication (password / qr code)
- Homepage
- Misc. instance/user infos
- Homework
- Timetable
- Grades
- Attachments
- Agenda
- Notifications
- Info (& polls)
- Discussions
- Lesson details
- Digital textbooks
- Disconnection
- School list based on geolocation

## Contributing
Every unexcepted fail (such as serde parsing error, etc) should panic.
Excepted fails like invalid credentials, etc shouldn't.

The `req/` and `resp/` contain sample data from the network communications between the website and the server.

- `cargo run (-r)` runs a CLI that can use mock data, connect to the demo pronote server or use real credentials. For a list of available commands, type `help`.
- `cargo test (-r)` check if all the client methods work on the sample data without panicking.

*A note on AI*: AI use is allowed but all the code must be handwritten and understandable.

## Legal notice
Index Education [indicated](https://www.index-education.com/fr/article-1760-recommandations-relatives-a-l-application-papillon.php) that as long as an unofficial app uses credentials that the user provided voluntarily and isn't commercial, it isn't prohibited.
> Cette pratique n'est pas illégale en soi, car justifiée par le fait que l'utilisateur renseigne volontairement ses informations de connexion et aussi par le fait que \[l'application (Papillon)] n'a pas de finalité commerciale.

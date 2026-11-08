# pronote-rs

> Ce projet n'est pas affilié avec PRONOTE ou Index Education

Une librairie non officielle en Rust qui interagit avec le service scolaire PRONOTE.
An unofficial rust crate to interact with the french PRONOTE school website.

## Fonctionnalités
- Authentication (mot de passe / qr code)
- Page d'accueil
- Informations sur l'établissement/utilisateur
- Devoirs
- Emploi du temps
- Notes
- Pièces jointes
- Agenda
- Notifications
- Informations (& sondages)
- Discussions
- Contenu du cours
- Ressources numériques
- Droit à la déconnexion
- Liste des établissements en fonction de la géolocalisation

## Contribuer
Toute erreur de la librairie (réponse du serveur mal comprise, etc) doit paniquer.
Les erreurs possibles commes mauvais mot de passe, etc devraient à la place retourner une erreur.

Les dossiers `req/` and `resp/` contiennent des données factices interceptées depuis les communications réseau entre le site web et le serveur.

- `cargo run (-r)` lance une application dans le terminal qui peut utiliser les données factices, se connecter au serveur démo de PRONOTE ou utiliser un vrai serveur avec utilisateur/mot de passe. Pour voir une liste des commandes disponible, tapez `help`.
- `cargo test (-r)` vérifie si toutes les méthodes client marchent sur les données factices sans paniquer.

*A propos de l'IA*: l'usage de l'IA est autorisé MAIS tout le code doit être écrit à la main et compréhensible.

## Note sur PRONOTE
Index Education [a indiqué](https://www.index-education.com/fr/article-1760-recommandations-relatives-a-l-application-papillon.php) que tant qu'une application PRONOTE non officielle utilise un mot de passe renseigné volontairement par l'utilisateur, et qu'elle n'a aucune forme de monétisation, ce n'est pas interdit.
> Cette pratique n'est pas illégale en soi, car justifiée par le fait que l'utilisateur renseigne volontairement ses informations de connexion et aussi par le fait que \[l'application (Papillon)] n'a pas de finalité commerciale.


## lib-email
lib-email bruker MAP sin [map-emailer](https://github.com/Mattilsynet/map-mailer/tree/master) for å sende epost til en bruker. Denne lib'n eksponerer kun funksjonen ```send_formattert_epost()``` som vil sende en ferdig HTML formattert epost til én mottaker, med tittel og innhold spesifisert som input. 

lib-email forholder seg til JSON når den legger data på NATS subjektet i map-mailer. Typene er bygget utifra email.proto, men protobuf brukes ikke i denne lib'n.

### Requirements
For at lib-email skal fungere må du prosjektet du henter lib-email inn i ha tilgang til å skrive til NATS subjektet "map-mailer".

Se [MAP sin dokumentasjon](https://github.com/Mattilsynet/map-mailer/tree/master) for informasjon om hvordan du får tilgang på NATS-subjektet.
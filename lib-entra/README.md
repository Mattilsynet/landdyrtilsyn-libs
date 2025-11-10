
# lib-entra
lib-entra eksponerer noen funksjoner som gjør det mulig å hente informasjon om ansatte i Mattilsynet fra Microsoft Entra gjennom ms graph-API.

## Hva kreves for å bruke lib-entra
For å bruke funksjonene i lib-entra må du sette opp tjenesten din til å ha korrekte rettigheter (scope) i Entra. Dette settes via terraform-modulen *map-tf-cloudrun*. Finn relevant service i [landdyrtilsyn-ops](https://github.com/Mattilsynet/landdyrtilsyn-ops) og sjekk dokumentasjon for [map-tf-cloudrun](https://github.com/Mattilsynet/map-tf-cloudrun) for å forstå hvordan du setter opp *delegated* eller *application* permission. 

lib-entra er delt opp i to: ```lib_entra::application_permission``` og ```lib_entra::delegated_permission```. Dette gir tilgang til funksjonalitet som er støtte av de to scopene.

For mer informasjon om Microsoft sitt graph API, sjekk [Infrastruktur sin oversikt](https://mattilsynet.atlassian.net/wiki/spaces/AU/pages/1238990858/Brukerinformasjon+EntraID).
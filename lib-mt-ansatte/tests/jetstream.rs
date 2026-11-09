use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use async_nats::jetstream::{Context, stream};
use lib_mt_ansatte::{Error, MtAnsatte, Result};
use uuid::Uuid;

const STREAM: &str = "ansatte";

struct NatsServer {
    child: Child,
    config_path: PathBuf,
    store_dir: PathBuf,
    url: String,
}

impl NatsServer {
    fn start() -> Option<Self> {
        match Command::new("nats-server")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => panic!("failed to execute nats-server: {error}"),
        }

        let name = format!("lib-mt-ansatte-{}", Uuid::new_v4());
        let store_dir = std::env::temp_dir().join(format!("{name}-js"));
        std::fs::create_dir_all(&store_dir).expect("create jetstream store dir");
        let config_path = std::env::temp_dir().join(format!("{name}.conf"));
        let config = format!(
            "jetstream {{ store_dir: '{}' }}\nmax_payload: 8388608\n",
            store_dir.display()
        );
        std::fs::write(&config_path, config).expect("write temporary nats-server config");

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve ephemeral port");
        let port = listener.local_addr().expect("read ephemeral port").port();
        drop(listener);

        let child = Command::new("nats-server")
            .arg("--addr")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .arg("--config")
            .arg(&config_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|error| {
                let _ = std::fs::remove_file(&config_path);
                let _ = std::fs::remove_dir_all(&store_dir);
                panic!("start nats-server: {error}");
            });

        Some(Self {
            child,
            config_path,
            store_dir,
            url: format!("nats://127.0.0.1:{port}"),
        })
    }
}

impl Drop for NatsServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.config_path);
        let _ = std::fs::remove_dir_all(&self.store_dir);
    }
}

async fn connect(url: &str) -> async_nats::Client {
    for _ in 0..50 {
        if let Ok(client) = async_nats::connect(url).await {
            return client;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("nats-server did not become ready");
}

async fn jetstream(server: &NatsServer) -> (MtAnsatte, Context) {
    let client = connect(&server.url).await;
    let context = async_nats::jetstream::new(client.clone());
    context
        .get_or_create_stream(stream::Config {
            name: STREAM.to_string(),
            subjects: vec!["ansatte.>".to_string()],
            storage: stream::StorageType::Memory,
            ..Default::default()
        })
        .await
        .expect("create stream");
    (MtAnsatte::new(client), context)
}

async fn publiser(context: &Context, subject: &str, payload: serde_json::Value) {
    context
        .publish(
            subject.to_string(),
            serde_json::to_vec(&payload).expect("serialize").into(),
        )
        .await
        .expect("publish")
        .await
        .expect("publish ack");
}

fn payload(navn: Option<&str>, enhet: Option<&str>, enabled: bool) -> serde_json::Value {
    serde_json::json!({
        "accountEnabled": enabled,
        "displayName": navn,
        "department": enhet,
        "employeeId": "99990000",
        "ignoredField": "verdi",
    })
}

#[tokio::test]
async fn henter_ansattnavn() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Test Person En".to_string())
    );
}

#[tokio::test]
async fn henter_enhetsnavn() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;

    assert_eq!(
        ansatte.enhet_navn("M10000").await.unwrap(),
        Some("Seksjon En".to_string())
    );
}

#[tokio::test]
async fn hoyeste_sekvens_vinner_pa_samme_subject() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    let subject = "ansatte.M10000.99990001";
    publiser(
        &context,
        subject,
        payload(Some("Gammelt Navn"), Some("Seksjon En"), true),
    )
    .await;
    publiser(
        &context,
        subject,
        payload(Some("Nytt Navn"), Some("Seksjon En"), true),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Nytt Navn".to_string())
    );
}

#[tokio::test]
async fn hoyeste_sekvens_vinner_pa_tvers_av_enheter() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Gammelt Navn"), Some("Seksjon En"), true),
    )
    .await;
    publiser(
        &context,
        "ansatte.M20000.99990001",
        payload(Some("Nytt Navn"), Some("Seksjon To"), true),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Nytt Navn".to_string())
    );
}

#[tokio::test]
async fn nyere_ukjent_subject_vinner() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Gammelt Navn"), Some("Seksjon En"), true),
    )
    .await;
    publiser(
        &context,
        "ansatte.ukjent.99990001",
        payload(Some("Uten Enhet"), None, true),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Uten Enhet".to_string())
    );
}

#[tokio::test]
async fn deaktivert_ansatt_kan_slaas_opp() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), false),
    )
    .await;

    assert_eq!(
        ansatte.ansatt_navn("99990001").await.unwrap(),
        Some("Test Person En".to_string())
    );
}

#[tokio::test]
async fn nyeste_melding_uten_navn_gir_none() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    let subject = "ansatte.M10000.99990001";
    publiser(
        &context,
        subject,
        payload(Some("Gammelt Navn"), Some("Seksjon En"), true),
    )
    .await;
    publiser(&context, subject, payload(None, Some("Seksjon En"), true)).await;

    assert_eq!(ansatte.ansatt_navn("99990001").await.unwrap(), None);
}

#[tokio::test]
async fn ingen_treff_gir_none() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;

    assert_eq!(ansatte.ansatt_navn("99990099").await.unwrap(), None);
    assert_eq!(ansatte.enhet_navn("M99999").await.unwrap(), None);
}

#[tokio::test]
async fn manglende_stream_gir_feil() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let client = connect(&server.url).await;
    let ansatte = MtAnsatte::new(client);

    let result: Result<Option<String>> = ansatte.ansatt_navn("99990001").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn ugyldig_ansatt_id_avvises_for_nats() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let client = connect(&server.url).await;
    let ansatte = MtAnsatte::new(client);

    let result: Result<Option<String>> = ansatte.ansatt_navn("17912.*").await;
    assert!(matches!(result, Err(Error::InvalidId(_))));
}

#[tokio::test]
async fn ugyldig_payload_gir_feil() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    context
        .publish("ansatte.M10000.99990001", "ikke-json".into())
        .await
        .expect("publish")
        .await
        .expect("publish ack");

    let result: Result<Option<String>> = ansatte.ansatt_navn("99990001").await;
    assert!(matches!(result, Err(Error::Payload(_))));
}

async fn publiser_flyttemarkor(context: &Context, subject: &str) {
    let mut headers = async_nats::HeaderMap::new();
    headers.insert("deleted", "true");
    context
        .publish_with_headers(subject.to_string(), headers, "".into())
        .await
        .expect("publish")
        .await
        .expect("publish ack");
}

#[tokio::test]
async fn henter_hele_ansattposten() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M42210.99990001",
        serde_json::json!({
            "accountEnabled": true,
            "displayName": "Test Person En",
            "department": "seksjon en",
            "employeeId": "99990001",
            "onPremisesExtensionAttributes": { "extensionAttribute6": "M42210" },
        }),
    )
    .await;

    let ansatt = ansatte.ansatt("99990001").await.unwrap().unwrap();
    assert_eq!(ansatt.account_enabled, Some(true));
    assert_eq!(ansatt.display_name.as_deref(), Some("Test Person En"));
    assert_eq!(ansatt.department.as_deref(), Some("seksjon en"));
    assert_eq!(
        ansatt
            .on_premises_extension_attributes
            .and_then(|attributter| attributter.extension_attribute6)
            .as_deref(),
        Some("M42210")
    );
    assert_eq!(ansatte.ansatt("99990099").await.unwrap(), None);
}

#[tokio::test]
async fn ansatt_folger_flytting_til_ny_enhet() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;
    publiser_flyttemarkor(&context, "ansatte.M10000.99990001").await;
    publiser(
        &context,
        "ansatte.M20000.99990001",
        payload(Some("Test Person En"), Some("Seksjon To"), true),
    )
    .await;

    let ansatt = ansatte.ansatt("99990001").await.unwrap().unwrap();
    assert_eq!(ansatt.department.as_deref(), Some("Seksjon To"));
}

#[tokio::test]
async fn flyttemarkor_som_nyeste_ansattpost_gir_feil() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;
    publiser_flyttemarkor(&context, "ansatte.M10000.99990001").await;

    assert!(ansatte.ansatt("99990001").await.is_err());
    assert!(ansatte.ansatt_navn("99990001").await.is_err());
}

#[tokio::test]
async fn enhetsnavn_finnes_selv_om_noen_har_flyttet_ut() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;
    publiser(
        &context,
        "ansatte.M10000.99990002",
        payload(Some("Test Person To"), Some("Seksjon En"), true),
    )
    .await;
    publiser_flyttemarkor(&context, "ansatte.M10000.99990002").await;
    publiser(
        &context,
        "ansatte.M20000.99990002",
        payload(Some("Test Person To"), Some("Seksjon To"), true),
    )
    .await;

    assert_eq!(
        ansatte.enhet_navn("M10000").await.unwrap(),
        Some("Seksjon En".to_string())
    );
}

#[tokio::test]
async fn enhet_der_alle_har_flyttet_ut_gir_none() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    publiser(
        &context,
        "ansatte.M10000.99990001",
        payload(Some("Test Person En"), Some("Seksjon En"), true),
    )
    .await;
    publiser_flyttemarkor(&context, "ansatte.M10000.99990001").await;

    assert_eq!(ansatte.enhet_navn("M10000").await.unwrap(), None);
}

fn post(nr: &str, koststed: &str, enhet: &str, enabled: bool) -> serde_json::Value {
    serde_json::json!({
        "accountEnabled": enabled,
        "displayName": format!("Test Person {nr}"),
        "employeeId": nr,
        "department": enhet,
        "onPremisesExtensionAttributes": { "extensionAttribute6": koststed },
    })
}

async fn organisasjon(context: &Context) {
    publiser(
        context,
        "ansatte.M42200.99990001",
        post("99990001", "M42200", "avdeling landdyr", true),
    )
    .await;
    publiser(
        context,
        "ansatte.M42210.99990002",
        post("99990002", "M42210", "seksjon nord", true),
    )
    .await;
    publiser(
        context,
        "ansatte.M42210.99990003",
        post("99990003", "M42210", "seksjon nord", true),
    )
    .await;
    publiser_flyttemarkor(context, "ansatte.M42210.99990003").await;
    publiser(
        context,
        "ansatte.M42300.99990003",
        post("99990003", "M42300", "avdeling slakteri", true),
    )
    .await;
    publiser(
        context,
        "ansatte.M42220.99990004",
        post("99990004", "M42220", "seksjon midt", false),
    )
    .await;
}

fn ansattnumre(ansatte: &[lib_mt_ansatte::Ansatt]) -> Vec<&str> {
    ansatte
        .iter()
        .filter_map(|ansatt| ansatt.employee_id.as_deref())
        .collect()
}

#[tokio::test]
async fn alle_ansatte_gir_nyeste_aktive_post_per_ansatt() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    organisasjon(&context).await;

    let alle = ansatte.alle_ansatte().await.unwrap();

    assert_eq!(ansattnumre(&alle), ["99990001", "99990002", "99990003"]);
    assert_eq!(alle[2].koststed(), Some("M42300"));
}

#[tokio::test]
async fn ansatte_paa_koststed_og_i_avdeling() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    organisasjon(&context).await;

    assert_eq!(
        ansattnumre(&ansatte.ansatte_paa_koststed("M42210").await.unwrap()),
        ["99990002"]
    );
    assert_eq!(
        ansattnumre(&ansatte.ansatte_i_avdeling("M42200").await.unwrap()),
        ["99990001", "99990002"]
    );
}

#[tokio::test]
async fn enheter_med_aktive_ansatte() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    organisasjon(&context).await;

    let enheter: Vec<(String, String)> = ansatte
        .enheter()
        .await
        .unwrap()
        .into_iter()
        .map(|enhet| (enhet.kode, enhet.navn))
        .collect();

    assert_eq!(
        enheter,
        [
            ("M42200".to_string(), "avdeling landdyr".to_string()),
            ("M42210".to_string(), "seksjon nord".to_string()),
            ("M42300".to_string(), "avdeling slakteri".to_string()),
        ]
    );
}

fn post_med_leder(nr: &str, koststed: &str, leder: &str) -> serde_json::Value {
    serde_json::json!({
        "accountEnabled": true,
        "displayName": format!("Test Person {nr}"),
        "employeeId": nr,
        "department": koststed,
        "onPremisesExtensionAttributes": {
            "extensionAttribute6": koststed,
            "extensionAttribute8": leder,
        },
    })
}

#[tokio::test]
async fn leder_for_seksjon_og_avdeling() {
    let Some(server) = NatsServer::start() else {
        eprintln!("skipping: nats-server is not installed");
        return;
    };
    let (ansatte, context) = jetstream(&server).await;
    for (nr, koststed, leder) in [
        ("99990001", "M42000", "99990000"),
        ("99990002", "M42200", "99990001"),
        ("99990003", "M42200", "99990001"),
        ("99990004", "M42210", "99990002"),
        ("99990005", "M42210", "99990004"),
        ("99990006", "M42210", "99990004"),
    ] {
        publiser(
            &context,
            &format!("ansatte.{koststed}.{nr}"),
            post_med_leder(nr, koststed, leder),
        )
        .await;
    }

    assert_eq!(
        ansattnumre(&ansatte.ledere_for_koststed("M42210").await.unwrap()),
        ["99990004"]
    );
    assert_eq!(
        ansattnumre(&ansatte.ledere_for_koststed("M42200").await.unwrap()),
        ["99990002"]
    );
    assert!(
        ansatte
            .ledere_for_koststed("M42220")
            .await
            .unwrap()
            .is_empty()
    );
}

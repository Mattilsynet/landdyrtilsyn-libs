use crate::error::{Error, Result};
use async_nats::jetstream::consumer::PullConsumer;
use async_nats::jetstream::stream::{self, Source, Stream};
use async_nats::jetstream::{consumer, Context, Message};
use futures::TryStreamExt;

pub async fn get_or_create_durable_consumer(
    context: &Context,
    stream_name: &str,
    filter_subject: String,
    name: &str,
    durable_name: &str,
) -> Result<consumer::PullConsumer> {
    let consumer: consumer::PullConsumer = context
        .get_stream(stream_name)
        .await?
        .get_or_create_consumer(
            name,
            consumer::pull::Config {
                durable_name: Some(durable_name.to_string()),
                deliver_policy: consumer::DeliverPolicy::All,
                filter_subject,
                ..Default::default()
            },
        )
        .await?;
    Ok(consumer)
}

pub async fn get_or_create_workque_stream(
    context: &Context,
    stream_name: &str,
    subjects: Vec<String>,
    num_replicas: usize,
) -> Result<Stream> {
    let stream = context
        .get_or_create_stream(stream::Config {
            name: stream_name.to_string(),
            subjects,
            num_replicas,
            retention: stream::RetentionPolicy::WorkQueue,
            discard: stream::DiscardPolicy::New,
            max_messages: 10000,
            ..Default::default()
        })
        .await?;
    Ok(stream)
}

pub async fn create_ephemeral_consumer_last_per_subject(
    context: &Context,
    stream_name: &str,
    filter_subject: String,
) -> Result<consumer::PullConsumer> {
    let consumer: consumer::PullConsumer = context
        .get_stream(stream_name)
        .await?
        .get_or_create_consumer(
            "bm_api_ephemeral_consumer_last",
            consumer::pull::Config {
                deliver_policy: consumer::DeliverPolicy::LastPerSubject,
                filter_subject,
                ..Default::default()
            },
        )
        .await?;
    Ok(consumer)
}

pub async fn create_ephemeral_consumer_all_per_subject(
    context: &Context,
    stream_name: &str,
    filter_subject: String,
) -> Result<consumer::PullConsumer> {
    let consumer: consumer::PullConsumer = context
        .get_stream(stream_name)
        .await?
        .get_or_create_consumer(
            "bm_api_ephemeral_consumer_all",
            consumer::pull::Config {
                deliver_policy: consumer::DeliverPolicy::All,
                filter_subject,
                ..Default::default()
            },
        )
        .await?;
    Ok(consumer)
}

pub async fn get_or_create_bm_sourced_stream(context: &Context) -> Result<Stream> {
    let stream = context
        .get_or_create_stream(stream::Config {
            name: "bekymringsmeldinger_all".to_string(),
            num_replicas: 3,
            subjects: vec![],
            // Configure sources
            sources: vec![
                Source {
                    name: "bekymringsmeldinger_rodtkjott".to_string(),
                    filter_subject: Some("bekymringsmeldinger.rodtkjott.v1.>".to_string()),
                    ..Default::default()
                },
                Source {
                    name: "bekymringsmeldinger_publikum".to_string(),
                    filter_subject: Some("bekymringsmeldinger.publikum.v1.>".to_string()),
                    ..Default::default()
                },
            ]
            .into(),
            ..Default::default()
        })
        .await?;
    Ok(stream)
}

pub async fn get_consumer_from_stream(
    jetstream: &Context,
    consumer_name: &str,
    stream_name: &str,
) -> Result<PullConsumer> {
    let consumer: PullConsumer = jetstream
        .get_consumer_from_stream(consumer_name, stream_name)
        .await?;
    Ok(consumer)
}

pub async fn get_last_message_from_subject(
    jetstream: &Context,
    subject: String,
) -> Result<Message> {
    let consumer =
        create_ephemeral_consumer_last_per_subject(jetstream, "bekymringsmeldinger_all", subject)
            .await?;

    let mut messages = consumer.fetch().max_messages(1).messages().await?;

    if let Ok(Some(message)) = messages
        .try_next()
        .await
        .map_err(|err| Error::FetchError(err.to_string()))
    {
        Ok(message)
    } else {
        Err(Error::NotFoundError("No message found".to_string()))
    }
}

pub async fn get_all_messages_from_bm_stream_subject(
    jetstream: &Context,
    subject: String,
) -> Result<Vec<Message>> {
    let mut all_messages = Vec::new();
    let consumer =
        create_ephemeral_consumer_all_per_subject(jetstream, "bekymringsmeldinger_all", subject)
            .await?;

    loop {
        let mut messages = consumer.fetch().max_messages(5).messages().await?;

        let mut found_messages = false;

        while let Ok(Some(message)) = messages.try_next().await {
            found_messages = true;
            all_messages.push(message);
        }

        if !found_messages {
            break;
        }
    }
    Ok(all_messages)
}

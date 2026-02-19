use std::io::Cursor;
use std::path::Path;

use async_nats::jetstream::Context;
use async_nats::jetstream::object_store;
use async_nats::jetstream::object_store::ObjectStore;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{Error, Result};

pub async fn get_or_create_object_store(
    context: &Context,
    config: object_store::Config,
) -> Result<ObjectStore> {
    match context.create_object_store(config.clone()).await {
        Ok(store) => Ok(store),
        Err(create_err) => match context.get_object_store(&config.bucket).await {
            Ok(store) => Ok(store),
            Err(get_err) => Err(Error::ConfigError(format!(
                "Failed to create or get object store: create error: {}; get error: {}",
                create_err, get_err
            ))),
        },
    }
}

pub async fn store_bytes(
    store: &ObjectStore,
    object_name: &str,
    payload: &[u8],
) -> Result<object_store::ObjectInfo> {
    let mut cursor = Cursor::new(payload);
    store
        .put(object_name, &mut cursor)
        .await
        .map_err(|err| Error::PublishError(err.to_string()))
}

pub async fn store_file_path<P: AsRef<Path>>(
    store: &ObjectStore,
    object_name: &str,
    path: P,
) -> Result<object_store::ObjectInfo> {
    let mut file = File::open(path)
        .await
        .map_err(|err| Error::FetchError(err.to_string()))?;
    store
        .put(object_name, &mut file)
        .await
        .map_err(|err| Error::PublishError(err.to_string()))
}

pub async fn fetch_bytes(store: &ObjectStore, object_name: &str) -> Result<Vec<u8>> {
    let mut object = store
        .get(object_name)
        .await
        .map_err(|err| match err.kind() {
            object_store::GetErrorKind::NotFound => Error::NotFoundError(err.to_string()),
            _ => Error::FetchError(err.to_string()),
        })?;
    let mut buffer = Vec::new();
    object
        .read_to_end(&mut buffer)
        .await
        .map_err(|err| Error::FetchError(err.to_string()))?;
    Ok(buffer)
}

pub async fn fetch_to_file_path<P: AsRef<Path>>(
    store: &ObjectStore,
    object_name: &str,
    destination: P,
) -> Result<()> {
    let mut object = store
        .get(object_name)
        .await
        .map_err(|err| match err.kind() {
            object_store::GetErrorKind::NotFound => Error::NotFoundError(err.to_string()),
            _ => Error::FetchError(err.to_string()),
        })?;
    let mut file = File::create(destination)
        .await
        .map_err(|err| Error::FetchError(err.to_string()))?;
    tokio::io::copy(&mut object, &mut file)
        .await
        .map_err(|err| Error::FetchError(err.to_string()))?;
    file.flush()
        .await
        .map_err(|err| Error::FetchError(err.to_string()))?;
    Ok(())
}

pub async fn delete_object(store: &ObjectStore, object_name: &str) -> Result<()> {
    store
        .delete(object_name)
        .await
        .map_err(|err| match err.kind() {
            object_store::DeleteErrorKind::NotFound => Error::NotFoundError(err.to_string()),
            _ => Error::PublishError(err.to_string()),
        })
}

pub async fn object_info(
    store: &ObjectStore,
    object_name: &str,
) -> Result<object_store::ObjectInfo> {
    store
        .info(object_name)
        .await
        .map_err(|err| match err.kind() {
            object_store::InfoErrorKind::NotFound => Error::NotFoundError(err.to_string()),
            _ => Error::FetchError(err.to_string()),
        })
}

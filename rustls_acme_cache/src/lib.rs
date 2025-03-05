use std::future::Future;

use async_trait::async_trait;
use aws_sdk_s3::Client;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::put_object::PutObjectError;
use aws_sdk_s3::primitives::{ByteStream, ByteStreamError};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use rustls_acme::{AccountCache, CertCache};
use sha2::{Digest, Sha256};

pub struct AcmeS3Cache {
    bucket: String,
    prefix: String,
}


impl AcmeS3Cache {
    pub fn new(bucket: String, prefix: String) -> Self {
        Self { bucket, prefix }
    }

    async fn use_client<T, E, Fut>(f: impl FnOnce(Client) -> Fut) -> Result<T, E>
        where Fut: Future<Output=Result<T, E>>
    {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        f(client).await
    }

    async fn get_client() -> Client
    {
        let config = aws_config::load_from_env().await;
        Client::new(&config)
    }

    //Copied directly from DirCache, for the most part
    fn cached_cert_file_name(domains: &[String], directory_url: impl AsRef<str>) -> String {
        let mut ctx = Sha256::default();
        for domain in domains {
            ctx.update(domain.as_bytes());
            ctx.update(&[0])
        }
        ctx.update(directory_url.as_ref().as_bytes());
        let hash = BASE64_URL_SAFE_NO_PAD.encode(ctx.finalize());
        format!("cached_cert_{}", hash)
    }
}


#[derive(Debug)]
pub enum MyErrors {
    PutObjectError(SdkError<PutObjectError>),
    GetObjectError(SdkError<GetObjectError>),
    ByteStreamError(ByteStreamError),
}

#[async_trait]
impl CertCache for AcmeS3Cache {
    type EC = MyErrors;

    async fn load_cert(&self, domains: &[String], directory_url: &str) -> Result<Option<Vec<u8>>, Self::EC> {
        let file_name = Self::cached_cert_file_name(&domains, directory_url);
        let get_object_output = Self::get_client().await
            .get_object()
            .bucket(&self.bucket)
            .key(&format!("{}/{}", &self.prefix, file_name))
            .send().await
            .map_err(MyErrors::GetObjectError)?;

        get_object_output.body.collect().await.map(|aggregated_bytes| aggregated_bytes.to_vec())
            .map(|bytes| if bytes.len() == 0 { None } else { Some(bytes) })
            .map_err(MyErrors::ByteStreamError)
    }

    //note for posterity, written mostly by co-pilot
    async fn store_cert(&self, domains: &[String], directory_url: &str, cert: &[u8]) -> Result<(), Self::EC> {
        let file_name = Self::cached_cert_file_name(&domains, directory_url);
        Self::use_client(|client| {
            client.put_object()
                .bucket(&self.bucket)
                .key(&format!("{}/{}", &self.prefix, file_name))
                .body(ByteStream::from(Vec::from(cert)))
                .send()
        }).await.map(|_| ())
            .map_err(MyErrors::PutObjectError)
    }
}

pub struct NoAccountAcmeS3Cache;

#[async_trait]
impl AccountCache for NoAccountAcmeS3Cache {
    type EA = MyErrors;

    async fn load_account(&self, _contact: &[String], _directory_url: &str) -> Result<Option<Vec<u8>>, Self::EA> {
        tracing::info!("no account cache configured, could not load account");
        Ok(None)
    }

    async fn store_account(&self, _contact: &[String], _directory_url: &str, _account: &[u8]) -> Result<(), Self::EA> {
        tracing::info!("no account cache configured, could not store account");
        Ok(())
    }
}

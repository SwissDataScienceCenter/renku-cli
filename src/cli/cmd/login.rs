use super::Context;
use crate::cli::sink::Error as SinkError;
use crate::httpclient::Error as HttpError;
use clap::Parser;
use openidconnect::core::*;
use openidconnect::reqwest::async_http_client;
use openidconnect::*;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Performs a login
#[derive(Parser, Debug, PartialEq)]
pub struct Input {}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("An http error occurred: {}", source))]
    HttpClient { source: HttpError },

    #[snafu(display("Error writing data: {}", source))]
    WriteResult { source: SinkError },
}

// Obtain the device_authorization_url from the OIDC metadata provider.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct DeviceEndpointProviderMetadata {
    device_authorization_endpoint: DeviceAuthorizationUrl,
}
impl AdditionalProviderMetadata for DeviceEndpointProviderMetadata {}
type DeviceProviderMetadata = ProviderMetadata<
    DeviceEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

impl Input {
    pub async fn exec(&self, ctx: &Context) -> Result<(), Error> {
        let issuer_url = IssuerUrl::new(
            ctx.renku_url()
                .join("auth/realms/Renku")
                .unwrap()
                .as_str()
                .to_string(),
        )
        .unwrap();

        let metadata = DeviceProviderMetadata::discover_async(issuer_url, async_http_client)
            .await
            .unwrap();

        println!(
            "device auth: {:?}",
            metadata.additional_metadata().device_authorization_endpoint
        );

        let client_id = ClientId::new("renku-cli".into());
        let device_url = metadata
            .additional_metadata()
            .device_authorization_endpoint
            .clone();
        let client = CoreClient::from_provider_metadata(metadata, client_id, None)
            .set_device_authorization_uri(device_url)
            .set_auth_type(AuthType::RequestBody);

        let details: CoreDeviceAuthorizationResponse = client
            .exchange_device_code()
            .unwrap()
            .request_async(async_http_client)
            .await
            .unwrap();
        println!("Fetching device code...");
        dbg!(&details);

        // Display the URL and user-code.
        println!(
            "Open this URL in your browser:\n{}\nand enter the code: {}",
            details.verification_uri_complete().unwrap().secret(),
            details.user_code().secret()
        );

        // poll for the token
        let token = client
            .exchange_device_access_token(&details)
            .request_async(async_http_client, tokio::time::sleep, None)
            .await
            .unwrap();

        println!("ID Token: {:?}", token.extra_fields().id_token());

        Ok(())
    }
}

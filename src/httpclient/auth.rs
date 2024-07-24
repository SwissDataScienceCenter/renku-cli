use std::fmt::Display;

use crate::data::renku_url::RenkuUrl;
use ::reqwest as rqw;
use iso8601_timestamp::{Duration, Timestamp};
use openidconnect::core::*;
use openidconnect::reqwest::async_http_client;
use openidconnect::*;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

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

pub type TokenResponse = StandardTokenResponse<
    IdTokenFields<
        EmptyAdditionalClaims,
        EmptyExtraTokenFields,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
        CoreJsonWebKeyType,
    >,
    CoreTokenType,
>;

pub fn access_token(r: &TokenResponse) -> String {
    r.access_token().secret().to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub created_at: Timestamp,
    pub response: TokenResponse,
}
impl Response {
    pub fn expires_at(&self) -> Option<Timestamp> {
        match &self.response.expires_in() {
            Some(d) => self
                .created_at
                .checked_add(Duration::nanoseconds(d.as_nanos() as i64)),
            None => None,
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.expires_at() {
            Some(end) => write!(f, "Login successful. Valid until {}.", end.format()),
            None => write!(f, "Login successful."),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCode {
    pub authorization_url: rqw::Url,
    pub user_code: String,
    metadata: DeviceProviderMetadata,
    device_auth_resp: CoreDeviceAuthorizationResponse,
}
impl Display for UserCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Please visit this url and authorize this application:\n{}\nUser-Code: {}",
            self.authorization_url, self.user_code
        )
    }
}

#[derive(Debug, Snafu)]
pub enum AuthError {
    #[snafu(display("Error reading url: {}", source))]
    UrlParse { source: url::ParseError },

    #[snafu(display("Error retrieving authentication provider metadata: {}", message))]
    Discover { message: String },

    #[snafu(display("Error exchanging tokens: {}", message))]
    CodeExchange { message: String },
}

const CLIENT_ID: &str = "renku-cli";
const REALM_PATH: &str = "auth/realms/Renku";

pub async fn get_user_code(renku_url: RenkuUrl) -> Result<UserCode, AuthError> {
    let issuer_url =
        IssuerUrl::from_url(renku_url.as_url().join(REALM_PATH).context(UrlParseSnafu)?);

    let metadata = DeviceProviderMetadata::discover_async(issuer_url, async_http_client)
        .await
        .map_err(|e| AuthError::Discover {
            message: format!("{}", e),
        })?;

    log::debug!(
        "device auth endpoint: {:?}",
        metadata.additional_metadata().device_authorization_endpoint
    );

    let device_url = metadata
        .additional_metadata()
        .device_authorization_endpoint
        .clone();
    let client =
        CoreClient::from_provider_metadata(metadata.clone(), ClientId::new(CLIENT_ID.into()), None)
            .set_device_authorization_uri(device_url)
            .set_auth_type(AuthType::RequestBody);

    let details: CoreDeviceAuthorizationResponse = client
        .exchange_device_code()
        .unwrap()
        .request_async(async_http_client)
        .await
        .unwrap();

    log::debug!("DeviceAuthResponse: {:?}", &details);

    let verify_url_str = details
        .verification_uri_complete()
        .map(|u| u.secret().to_owned())
        .unwrap_or(details.verification_uri().to_string());
    let verify_url: rqw::Url = rqw::Url::parse(&verify_url_str).context(UrlParseSnafu)?;
    Ok(UserCode {
        authorization_url: verify_url,
        user_code: details.user_code().secret().clone(),
        metadata: metadata.clone(),
        device_auth_resp: details,
    })
}

pub async fn poll_tokens(code: UserCode) -> Result<Response, AuthError> {
    let device_url = code
        .metadata
        .additional_metadata()
        .device_authorization_endpoint
        .clone();
    let client =
        CoreClient::from_provider_metadata(code.metadata, ClientId::new(CLIENT_ID.into()), None)
            .set_device_authorization_uri(device_url)
            .set_auth_type(AuthType::RequestBody);

    Ok(Response {
        created_at: Timestamp::now_utc(),
        response: client
            .exchange_device_access_token(&code.device_auth_resp)
            .request_async(async_http_client, tokio::time::sleep, None)
            .await
            .map_err(|e| AuthError::CodeExchange {
                message: format!("{}", e),
            })?,
    })
}

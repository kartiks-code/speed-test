use async_trait::async_trait;
use bytes::Bytes;
use futures::{Stream, future, future::BoxFuture, stream, future::TryFutureExt, future::FutureExt, stream::StreamExt};
use http_body_util::{combinators::BoxBody, Full};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{body::{Body, Incoming}, Request, Response, service::Service, Uri};
use percent_encoding::{utf8_percent_encode, AsciiSet};
use std::borrow::Cow;
use std::convert::{TryInto, Infallible};
use std::io::{ErrorKind, Read};
use std::error::Error;
use std::future::Future;
use std::fmt;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::str;
use std::str::FromStr;
use std::string::ToString;
use std::task::{Context, Poll};
use swagger::{ApiError, AuthData, BodyExt, Connector, DropContextService, Has, XSpanIdString};
use url::form_urlencoded;
use tower_service::Service as _;


use crate::models;
use crate::header;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
#[allow(dead_code)]
const FRAGMENT_ENCODE_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

/// This encode set is used for object IDs
///
/// Aside from the special characters defined in the `PATH_SEGMENT_ENCODE_SET`,
/// the vertical bar (|) is encoded.
#[allow(dead_code)]
const ID_ENCODE_SET: &AsciiSet = &FRAGMENT_ENCODE_SET.add(b'|');

use crate::{Api,
     AddPetResponse,
     FindPetsByStatusResponse,
     FindPetsByTagsResponse,
     UpdatePetResponse,
     DeletePetResponse,
     GetPetByIdResponse,
     UpdatePetWithFormResponse,
     UploadFileResponse,
     GetInventoryResponse,
     PlaceOrderResponse,
     DeleteOrderResponse,
     GetOrderByIdResponse,
     CreateUserResponse,
     CreateUsersWithListInputResponse,
     LoginUserResponse,
     LogoutUserResponse,
     DeleteUserResponse,
     GetUserByNameResponse,
     UpdateUserResponse
     };

/// Convert input into a base path, e.g. "http://example:123". Also checks the scheme as it goes.
fn into_base_path(input: impl TryInto<Uri, Error=hyper::http::uri::InvalidUri>, correct_scheme: Option<&'static str>) -> Result<String, ClientInitError> {
    // First convert to Uri, since a base path is a subset of Uri.
    let uri = input.try_into()?;

    let scheme = uri.scheme_str().ok_or(ClientInitError::InvalidScheme)?;

    // Check the scheme if necessary
    if let Some(correct_scheme) = correct_scheme {
        if scheme != correct_scheme {
            return Err(ClientInitError::InvalidScheme);
        }
    }

    let host = uri.host().ok_or(ClientInitError::MissingHost)?;
    let port = uri.port_u16().map(|x| format!(":{x}")).unwrap_or_default();
    Ok(format!("{scheme}://{host}{port}{}", uri.path().trim_end_matches('/')))
}

/// A client that implements the API by making HTTP calls out to a server.
pub struct Client<S, C> where
    S: Service<
           (Request<BoxBody<Bytes, Infallible>>, C)> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static
{
    /// Inner service
    client_service: S,

    /// Base path of the API
    base_path: String,

    /// Marker
    marker: PhantomData<fn(C)>,
}

impl<S, C> fmt::Debug for Client<S, C> where
    S: Service<
           (Request<BoxBody<Bytes, Infallible>>, C)> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ base_path: {} }}", self.base_path)
    }
}

impl<S, C> Clone for Client<S, C> where
    S: Service<
           (Request<BoxBody<Bytes, Infallible>>, C)> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static
{
    fn clone(&self) -> Self {
        Self {
            client_service: self.client_service.clone(),
            base_path: self.base_path.clone(),
            marker: PhantomData,
        }
    }
}

impl<Connector, C> Client<
    DropContextService<
        hyper_util::service::TowerToHyperService<
            hyper_util::client::legacy::Client<
                Connector,
                BoxBody<Bytes, Infallible>
            >
        >,
        C
    >,
    C
> where
    Connector: hyper_util::client::legacy::connect::Connect + Clone + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
{
    /// Create a client with a custom implementation of hyper::client::Connect.
    ///
    /// Intended for use with custom implementations of connect for e.g. protocol logging
    /// or similar functionality which requires wrapping the transport layer. When wrapping a TCP connection,
    /// this function should be used in conjunction with `swagger::Connector::builder()`.
    ///
    /// For ordinary tcp connections, prefer the use of `try_new_http`, `try_new_https`
    /// and `try_new_https_mutual`, to avoid introducing a dependency on the underlying transport layer.
    ///
    /// # Arguments
    ///
    /// * `base_path` - base path of the client API, i.e. "<http://www.my-api-implementation.com>"
    /// * `protocol` - Which protocol to use when constructing the request url, e.g. `Some("http")`
    /// * `connector` - Implementation of `hyper::client::Connect` to use for the client
    pub fn try_new_with_connector(
        base_path: &str,
        protocol: Option<&'static str>,
        connector: Connector,
    ) -> Result<Self, ClientInitError>
    {
        let client_service = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(connector);
        let client_service = DropContextService::new(hyper_util::service::TowerToHyperService::new(client_service));

        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, protocol)?,
            marker: PhantomData,
        })
    }
}

#[cfg(all(feature = "client-tls", any(target_os = "macos", target_os = "windows", target_os = "ios")))]
type HyperHttpsConnector = hyper_tls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>;

#[cfg(all(feature = "client-tls", not(any(target_os = "macos", target_os = "windows", target_os = "ios"))))]
type HyperHttpsConnector = hyper_openssl::client::legacy::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>;

#[derive(Debug, Clone)]
pub enum HyperClient {
    Http(hyper_util::client::legacy::Client<hyper_util::client::legacy::connect::HttpConnector, BoxBody<Bytes, Infallible>>),
    #[cfg(feature = "client-tls")]
    Https(hyper_util::client::legacy::Client<HyperHttpsConnector, BoxBody<Bytes, Infallible>>),
}

impl Service<Request<BoxBody<Bytes, Infallible>>> for HyperClient {
    type Response = Response<Incoming>;
    type Error = hyper_util::client::legacy::Error;
    type Future = hyper_util::client::legacy::ResponseFuture;

    fn call(&self, req: Request<BoxBody<Bytes, Infallible>>) -> Self::Future {
       match self {
          HyperClient::Http(client) => client.request(req),
          #[cfg(feature = "client-tls")]
          HyperClient::Https(client) => client.request(req),
       }
    }
}

impl<C> Client<DropContextService<HyperClient, C>, C> where
    C: Clone + Send + Sync + 'static,
{
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "<http://www.my-api-implementation.com>"
    pub fn try_new(
        base_path: &str,
    ) -> Result<Self, ClientInitError> {
        let uri = Uri::from_str(base_path)?;

        let scheme = uri.scheme_str().ok_or(ClientInitError::InvalidScheme)?;
        let scheme = scheme.to_ascii_lowercase();

        let connector = Connector::builder();

        let client_service = match scheme.as_str() {
            "http" => {
                HyperClient::Http(hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(connector.build()))
            },
            #[cfg(feature = "client-tls")]
            "https" => {
                let https_connector = connector
                    .https()
                    .build()
                    .map_err(ClientInitError::SslError)?;
                HyperClient::Https(hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(https_connector))
            },
            #[cfg(not(feature = "client-tls"))]
            "https" => {
                return Err(ClientInitError::TlsNotEnabled);
            },
            _ => {
                return Err(ClientInitError::InvalidScheme);
            }
        };

        let client_service = DropContextService::new(client_service);

        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, None)?,
            marker: PhantomData,
        })
    }
}

impl<C> Client<
    DropContextService<
        hyper_util::service::TowerToHyperService<
            hyper_util::client::legacy::Client<
                hyper_util::client::legacy::connect::HttpConnector,
                BoxBody<Bytes, Infallible>
            >
        >,
        C
    >,
    C
> where
    C: Clone + Send + Sync + 'static
{
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "<http://www.my-api-implementation.com>"
    pub fn try_new_http(
        base_path: &str,
    ) -> Result<Self, ClientInitError> {
        let http_connector = Connector::builder().build();

        Self::try_new_with_connector(base_path, Some("http"), http_connector)
    }
}

#[cfg(all(feature = "client-tls", any(target_os = "macos", target_os = "windows", target_os = "ios")))]
type HttpsConnector = hyper_tls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>;

#[cfg(all(feature = "client-tls", not(any(target_os = "macos", target_os = "windows", target_os = "ios"))))]
type HttpsConnector = hyper_openssl::client::legacy::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>;

#[cfg(feature = "client-tls")]
impl<C> Client<
    DropContextService<
        hyper_util::service::TowerToHyperService<
            hyper_util::client::legacy::Client<
                HttpsConnector,
                BoxBody<Bytes, Infallible>
            >
        >,
        C
    >,
    C
> where
    C: Clone + Send + Sync + 'static
{
    /// Create a client with a TLS connection to the server.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "<https://www.my-api-implementation.com>"
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
    pub fn try_new_https(base_path: &str) -> Result<Self, ClientInitError>
    {
        let https_connector = Connector::builder()
            .https()
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a TLS connection to the server using OpenSSL via swagger.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "<https://www.my-api-implementation.com>"
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https(base_path: &str) -> Result<Self, ClientInitError>
    {
        let https_connector = Connector::builder()
            .https()
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a TLS connection to the server using a pinned certificate.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "<https://www.my-api-implementation.com>"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https_pinned<CA>(
        base_path: &str,
        ca_certificate: CA,
    ) -> Result<Self, ClientInitError>
    where
        CA: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a mutually authenticated TLS connection to the server.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "<https://www.my-api-implementation.com>"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    /// * `client_key` - Path to the client private key
    /// * `client_certificate` - Path to the client's public certificate associated with the private key
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https_mutual<CA, K, D>(
        base_path: &str,
        ca_certificate: CA,
        client_key: K,
        client_certificate: D,
    ) -> Result<Self, ClientInitError>
    where
        CA: AsRef<Path>,
        K: AsRef<Path>,
        D: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .client_authentication(client_key, client_certificate)
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }
}

impl<S, C> Client<S, C> where
    S: Service<
           (Request<BoxBody<Bytes, Infallible>>, C)> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static
{
    /// Constructor for creating a `Client` by passing in a pre-made `hyper::service::Service` /
    /// `tower::Service`
    ///
    /// This allows adding custom wrappers around the underlying transport, for example for logging.
    pub fn try_new_with_client_service(
        client_service: S,
        base_path: &str,
    ) -> Result<Self, ClientInitError>
    {
        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, None)?,
            marker: PhantomData,
        })
    }
}

/// Error type failing to create a Client
#[derive(Debug)]
pub enum ClientInitError {
    /// Invalid URL Scheme
    InvalidScheme,

    /// Invalid URI
    InvalidUri(hyper::http::uri::InvalidUri),

    /// Missing Hostname
    MissingHost,

    /// HTTPS requested but TLS features not enabled
    TlsNotEnabled,

    /// SSL Connection Error
    #[cfg(all(feature = "client-tls", any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    SslError(native_tls::Error),

    /// SSL Connection Error
    #[cfg(all(feature = "client-tls", not(any(target_os = "macos", target_os = "windows", target_os = "ios"))))]
    SslError(openssl::error::ErrorStack),
}

impl From<hyper::http::uri::InvalidUri> for ClientInitError {
    fn from(err: hyper::http::uri::InvalidUri) -> ClientInitError {
        ClientInitError::InvalidUri(err)
    }
}

impl fmt::Display for ClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &dyn fmt::Debug = self;
        s.fmt(f)
    }
}

impl Error for ClientInitError {
    fn description(&self) -> &str {
        "Failed to produce a hyper client."
    }
}

#[allow(dead_code)]
fn body_from_string(s: String) -> BoxBody<Bytes, Infallible> {
    BoxBody::new(Full::new(Bytes::from(s)))
}

#[async_trait]
impl<S, C, B> Api<C> for Client<S, C> where
    S: Service<
       (Request<BoxBody<Bytes, Infallible>>, C),
       Response=Response<B>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Has<XSpanIdString> + Has<Option<AuthData>> + Clone + Send + Sync + 'static,
    B: hyper::body::Body + Send + 'static + Unpin,
    B::Data: Send,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{

    #[allow(clippy::vec_init_then_push)]
    async fn add_pet(
        &self,
        param_pet: models::Pet,
        context: &C) -> Result<AddPetResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/pet",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("POST")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        // Consumes form body

        // Consumes basic body
        // Body parameter
        let body = param_pet.as_xml();
        *request.body_mut() = body_from_string(body);

        let header = "application/json";
        request.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(header));

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::Bearer(ref bearer_header) => {
                    let header = match headers::Authorization::bearer(&bearer_header.to_string()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {e}")))
                    };
                    request.headers_mut().insert(
                        hyper::header::AUTHORIZATION,
                        header.0.encode());
                },
                _ => {}
            }
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Pet>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(AddPetResponse::SuccessfulOperation
                    (body)
                )
            }
            400 => {
                Ok(
                    AddPetResponse::InvalidInput
                )
            }
            422 => {
                Ok(
                    AddPetResponse::ValidationException
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(AddPetResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn find_pets_by_status(
        &self,
        param_status: Option<models::FindPetsByStatusStatusParameter>,
        context: &C) -> Result<FindPetsByStatusResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/pet/findByStatus",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_status) = param_status {
                query_string.append_pair("status",
                    &param_status.to_string());
            }
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::Bearer(ref bearer_header) => {
                    let header = match headers::Authorization::bearer(&bearer_header.to_string()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {e}")))
                    };
                    request.headers_mut().insert(
                        hyper::header::AUTHORIZATION,
                        header.0.encode());
                },
                _ => {}
            }
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<Vec<models::Pet>>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(FindPetsByStatusResponse::SuccessfulOperation
                    (body)
                )
            }
            400 => {
                Ok(
                    FindPetsByStatusResponse::InvalidStatusValue
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(FindPetsByStatusResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn find_pets_by_tags<'a>(
        &self,
        param_tags: Option<&'a Vec<String>>,
        context: &C) -> Result<FindPetsByTagsResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/pet/findByTags",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_tags) = param_tags {
                query_string.append_pair("tags",
                    &param_tags.iter().map(ToString::to_string).collect::<Vec<String>>().join(","));
            }
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::Bearer(ref bearer_header) => {
                    let header = match headers::Authorization::bearer(&bearer_header.to_string()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {e}")))
                    };
                    request.headers_mut().insert(
                        hyper::header::AUTHORIZATION,
                        header.0.encode());
                },
                _ => {}
            }
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<Vec<models::Pet>>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(FindPetsByTagsResponse::SuccessfulOperation
                    (body)
                )
            }
            400 => {
                Ok(
                    FindPetsByTagsResponse::InvalidTagValue
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(FindPetsByTagsResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn update_pet(
        &self,
        param_pet: models::Pet,
        context: &C) -> Result<UpdatePetResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/pet",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("PUT")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        // Consumes form body

        // Consumes basic body
        // Body parameter
        let body = param_pet.as_xml();
        *request.body_mut() = body_from_string(body);

        let header = "application/json";
        request.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(header));

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::Bearer(ref bearer_header) => {
                    let header = match headers::Authorization::bearer(&bearer_header.to_string()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {e}")))
                    };
                    request.headers_mut().insert(
                        hyper::header::AUTHORIZATION,
                        header.0.encode());
                },
                _ => {}
            }
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Pet>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(UpdatePetResponse::SuccessfulOperation
                    (body)
                )
            }
            400 => {
                Ok(
                    UpdatePetResponse::InvalidIDSupplied
                )
            }
            404 => {
                Ok(
                    UpdatePetResponse::PetNotFound
                )
            }
            422 => {
                Ok(
                    UpdatePetResponse::ValidationException
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(UpdatePetResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn delete_pet(
        &self,
        param_pet_id: i64,
        param_api_key: Option<String>,
        context: &C) -> Result<DeletePetResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/pet/{pet_id}",
            self.base_path
            ,pet_id=utf8_percent_encode(&param_pet_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::Bearer(ref bearer_header) => {
                    let header = match headers::Authorization::bearer(&bearer_header.to_string()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {e}")))
                    };
                    request.headers_mut().insert(
                        hyper::header::AUTHORIZATION,
                        header.0.encode());
                },
                _ => {}
            }
        }

        // Header parameters
        #[allow(clippy::single_match)]
        match param_api_key {
            Some(param_api_key) => {
        request.headers_mut().append(
            HeaderName::from_static("api_key"),
            #[allow(clippy::redundant_clone, clippy::clone_on_copy)]
            match header::IntoHeaderValue(param_api_key.clone()).try_into() {
                Ok(header) => header,
                Err(e) => {
                    return Err(ApiError(format!(
                        "Invalid header api_key - {e}")));
                },
            });
            },
            None => {}
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                Ok(
                    DeletePetResponse::SuccessfulOperation
                )
            }
            400 => {
                Ok(
                    DeletePetResponse::InvalidPetValue
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(DeletePetResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn get_pet_by_id(
        &self,
        param_pet_id: i64,
        context: &C) -> Result<GetPetByIdResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/pet/{pet_id}",
            self.base_path
            ,pet_id=utf8_percent_encode(&param_pet_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::Bearer(ref bearer_header) => {
                    let header = match headers::Authorization::bearer(&bearer_header.to_string()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {e}")))
                    };
                    request.headers_mut().insert(
                        hyper::header::AUTHORIZATION,
                        header.0.encode());
                },
                AuthData::ApiKey(ref api_key) => {
                    let header = match HeaderValue::from_str(api_key.as_str()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create header: {e}")))
                    };
                    request.headers_mut().insert(
                        HeaderName::from_static("api_key"),
                        header);
                },
                _ => {}
            }
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Pet>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(GetPetByIdResponse::SuccessfulOperation
                    (body)
                )
            }
            400 => {
                Ok(
                    GetPetByIdResponse::InvalidIDSupplied
                )
            }
            404 => {
                Ok(
                    GetPetByIdResponse::PetNotFound
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(GetPetByIdResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn update_pet_with_form(
        &self,
        param_pet_id: i64,
        param_name: Option<String>,
        param_status: Option<String>,
        context: &C) -> Result<UpdatePetWithFormResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/pet/{pet_id}",
            self.base_path
            ,pet_id=utf8_percent_encode(&param_pet_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_name) = param_name {
                query_string.append_pair("name",
                    &param_name);
            }
            if let Some(param_status) = param_status {
                query_string.append_pair("status",
                    &param_status);
            }
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("POST")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::Bearer(ref bearer_header) => {
                    let header = match headers::Authorization::bearer(&bearer_header.to_string()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {e}")))
                    };
                    request.headers_mut().insert(
                        hyper::header::AUTHORIZATION,
                        header.0.encode());
                },
                _ => {}
            }
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                Ok(
                    UpdatePetWithFormResponse::SuccessfullyUpdated
                )
            }
            400 => {
                Ok(
                    UpdatePetWithFormResponse::InvalidInput
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(UpdatePetWithFormResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn upload_file(
        &self,
        param_pet_id: i64,
        param_additional_metadata: Option<String>,
        param_body: Option<swagger::ByteArray>,
        context: &C) -> Result<UploadFileResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/pet/{pet_id}/uploadImage",
            self.base_path
            ,pet_id=utf8_percent_encode(&param_pet_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_additional_metadata) = param_additional_metadata {
                query_string.append_pair("additionalMetadata",
                    &param_additional_metadata);
            }
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("POST")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        // Consumes basic body
        // Body parameter
        if let Some(param_body) = param_body {
        let body = String::from_utf8(param_body.0).expect("Body was not valid UTF8");
        *request.body_mut() = body_from_string(body);
        }

        let header = "application/octet-stream";
        request.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(header));

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::Bearer(ref bearer_header) => {
                    let header = match headers::Authorization::bearer(&bearer_header.to_string()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create Authorization header: {e}")))
                    };
                    request.headers_mut().insert(
                        hyper::header::AUTHORIZATION,
                        header.0.encode());
                },
                _ => {}
            }
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::ApiResponse>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(UploadFileResponse::SuccessfulOperation
                    (body)
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(UploadFileResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn get_inventory(
        &self,
        context: &C) -> Result<GetInventoryResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/store/inventory",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            use headers::authorization::Credentials;
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                AuthData::ApiKey(ref api_key) => {
                    let header = match HeaderValue::from_str(api_key.as_str()) {
                        Ok(h) => h,
                        Err(e) => return Err(ApiError(format!("Unable to create header: {e}")))
                    };
                    request.headers_mut().insert(
                        HeaderName::from_static("api_key"),
                        header);
                },
                _ => {}
            }
        }

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<std::collections::HashMap<String, i32>>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(GetInventoryResponse::SuccessfulOperation
                    (body)
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(GetInventoryResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn place_order(
        &self,
        param_order: Option<models::Order>,
        context: &C) -> Result<PlaceOrderResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/store/order",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("POST")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        // Consumes form body

        // Consumes basic body
        // Body parameter
        if let Some(param_order) = param_order {
        let body = param_order.as_xml();
        *request.body_mut() = body_from_string(body);
        }

        let header = "application/json";
        request.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(header));

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Order>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(PlaceOrderResponse::SuccessfulOperation
                    (body)
                )
            }
            400 => {
                Ok(
                    PlaceOrderResponse::InvalidInput
                )
            }
            422 => {
                Ok(
                    PlaceOrderResponse::ValidationException
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(PlaceOrderResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn delete_order(
        &self,
        param_order_id: i64,
        context: &C) -> Result<DeleteOrderResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/store/order/{order_id}",
            self.base_path
            ,order_id=utf8_percent_encode(&param_order_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                Ok(
                    DeleteOrderResponse::SuccessfulOperation
                )
            }
            400 => {
                Ok(
                    DeleteOrderResponse::InvalidIDSupplied
                )
            }
            404 => {
                Ok(
                    DeleteOrderResponse::OrderNotFound
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(DeleteOrderResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn get_order_by_id(
        &self,
        param_order_id: i64,
        context: &C) -> Result<GetOrderByIdResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/store/order/{order_id}",
            self.base_path
            ,order_id=utf8_percent_encode(&param_order_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Order>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(GetOrderByIdResponse::SuccessfulOperation
                    (body)
                )
            }
            400 => {
                Ok(
                    GetOrderByIdResponse::InvalidIDSupplied
                )
            }
            404 => {
                Ok(
                    GetOrderByIdResponse::OrderNotFound
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(GetOrderByIdResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn create_user(
        &self,
        param_user: Option<models::User>,
        context: &C) -> Result<CreateUserResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/user",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("POST")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        // Consumes form body

        // Consumes basic body
        // Body parameter
        if let Some(param_user) = param_user {
        let body = param_user.as_xml();
        *request.body_mut() = body_from_string(body);
        }

        let header = "application/json";
        request.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(header));

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::User>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(CreateUserResponse::SuccessfulOperation
                    (body)
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(CreateUserResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn create_users_with_list_input<'a>(
        &self,
        param_user: Option<&'a Vec<models::User>>,
        context: &C) -> Result<CreateUsersWithListInputResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/user/createWithList",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("POST")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        // Consumes basic body
        // Body parameter
        if let Some(param_user) = param_user {
        let body = serde_json::to_string(&param_user).expect("impossible to fail to serialize");
        *request.body_mut() = body_from_string(body);
        }

        let header = "application/json";
        request.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(header));

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::User>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(CreateUsersWithListInputResponse::SuccessfulOperation
                    (body)
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(CreateUsersWithListInputResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn login_user(
        &self,
        param_username: Option<String>,
        param_password: Option<String>,
        context: &C) -> Result<LoginUserResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/user/login",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_username) = param_username {
                query_string.append_pair("username",
                    &param_username);
            }
            if let Some(param_password) = param_password {
                query_string.append_pair("password",
                    &param_password);
            }
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let response_x_rate_limit = match response.headers().get(HeaderName::from_static("x-rate-limit")) {
                    Some(response_x_rate_limit) => {
                        let response_x_rate_limit = response_x_rate_limit.clone();
                        let response_x_rate_limit = match TryInto::<header::IntoHeaderValue<i32>>::try_into(response_x_rate_limit) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header X-Rate-Limit for response 200 - {e}")));
                            },
                        };
                        Some(response_x_rate_limit.0)
                        },
                    None => None,
                };

                let response_x_expires_after = match response.headers().get(HeaderName::from_static("x-expires-after")) {
                    Some(response_x_expires_after) => {
                        let response_x_expires_after = response_x_expires_after.clone();
                        let response_x_expires_after = match TryInto::<header::IntoHeaderValue<chrono::DateTime::<chrono::Utc>>>::try_into(response_x_expires_after) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header X-Expires-After for response 200 - {e}")));
                            },
                        };
                        Some(response_x_expires_after.0)
                        },
                    None => None,
                };

                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                // ToDo: this will move to swagger-rs and become a standard From conversion trait
                // once https://github.com/RReverser/serde-xml-rs/pull/45 is accepted upstream
                let body = serde_xml_rs::from_str::<String>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(LoginUserResponse::SuccessfulOperation
                    {
                        body,
                        x_rate_limit: response_x_rate_limit,
                        x_expires_after: response_x_expires_after,
                    }
                )
            }
            400 => {
                Ok(
                    LoginUserResponse::InvalidUsername
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(LoginUserResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn logout_user(
        &self,
        context: &C) -> Result<LogoutUserResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/user/logout",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                Ok(
                    LogoutUserResponse::SuccessfulOperation
                )
            }
            0 => {
                Ok(
                    LogoutUserResponse::SuccessfulOperation_2
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn delete_user(
        &self,
        param_username: String,
        context: &C) -> Result<DeleteUserResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/user/{username}",
            self.base_path
            ,username=utf8_percent_encode(&param_username.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("DELETE")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                Ok(
                    DeleteUserResponse::SuccessfulOperation
                )
            }
            400 => {
                Ok(
                    DeleteUserResponse::InvalidUsernameSupplied
                )
            }
            404 => {
                Ok(
                    DeleteUserResponse::UserNotFound
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(DeleteUserResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn get_user_by_name(
        &self,
        param_username: String,
        context: &C) -> Result<GetUserByNameResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/user/{username}",
            self.base_path
            ,username=utf8_percent_encode(&param_username.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("GET")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::User>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(GetUserByNameResponse::SuccessfulOperation
                    (body)
                )
            }
            400 => {
                Ok(
                    GetUserByNameResponse::InvalidUsernameSupplied
                )
            }
            404 => {
                Ok(
                    GetUserByNameResponse::UserNotFound
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(GetUserByNameResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

    #[allow(clippy::vec_init_then_push)]
    async fn update_user(
        &self,
        param_username: String,
        param_user: Option<models::User>,
        context: &C) -> Result<UpdateUserResponse, ApiError>
    {
        let mut client_service = self.client_service.clone();
        #[allow(clippy::uninlined_format_args)]
        let mut uri = format!(
            "{}/api/v3/user/{username}",
            self.base_path
            ,username=utf8_percent_encode(&param_username.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {err}"))),
        };

        let mut request = match Request::builder()
            .method("PUT")
            .uri(uri)
            .body(BoxBody::new(http_body_util::Empty::new())) {
                Ok(req) => req,
                Err(e) => return Err(ApiError(format!("Unable to create request: {e}")))
        };

        // Consumes form body

        // Consumes basic body
        // Body parameter
        if let Some(param_user) = param_user {
        let body = param_user.as_xml();
        *request.body_mut() = body_from_string(body);
        }

        let header = "application/json";
        request.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(header));

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(HeaderName::from_static("x-span-id"), match header {
            Ok(h) => h,
            Err(e) => return Err(ApiError(format!("Unable to create X-Span ID header value: {e}")))
        });

        let response = client_service.call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {e}"))).await?;

        match response.status().as_u16() {
            200 => {
                Ok(
                    UpdateUserResponse::SuccessfulOperation
                )
            }
            0 => {
                let body = response.into_body();
                let body = http_body_util::BodyExt::collect(body)
                        .await
                        .map(|f| f.to_bytes().to_vec())
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e.into())))?;

                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {e}")))?;
                let body = serde_json::from_str::<models::Error>(body)
                    .map_err(|e| ApiError(format!("Response body did not match the schema: {e}")))?;


                Ok(UpdateUserResponse::UnexpectedError
                    (body)
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = http_body_util::BodyExt::collect(response.into_body())
                        .await
                        .map(|f| f.to_bytes().to_vec());
                Err(ApiError(format!("Unexpected response code {code}:\n{headers:?}\n\n{}",
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {e:?}>"),
                        },
                        Err(e) => format!("<Failed to read body: {}>", Into::<crate::ServiceError>::into(e)),
                    }
                )))
            }
        }
    }

}

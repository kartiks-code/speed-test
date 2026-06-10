use bytes::Bytes;
use futures::{future, future::BoxFuture, Stream, stream, future::FutureExt, stream::TryStreamExt};
use http_body_util::{combinators::BoxBody, Full};
use hyper::{body::{Body, Incoming}, HeaderMap, Request, Response, StatusCode};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use log::warn;
#[cfg(feature = "validate")]
use serde_valid::Validate;
#[allow(unused_imports)]
use std::convert::{TryFrom, TryInto};
use std::{convert::Infallible, error::Error};
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
use swagger::{ApiError, BodyExt, Has, RequestParser, XSpanIdString};
pub use swagger::auth::Authorization;
use swagger::auth::Scopes;
use url::form_urlencoded;

#[allow(unused_imports)]
use crate::{models, header, AuthenticationApi};

pub use crate::context;

type ServiceFuture = BoxFuture<'static, Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>>;

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

mod server_auth;

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(vec![
            r"^/api/v3/pet$",
            r"^/api/v3/pet/findByStatus$",
            r"^/api/v3/pet/findByTags$",
            r"^/api/v3/pet/(?P<petId>[^/?#]*)$",
            r"^/api/v3/pet/(?P<petId>[^/?#]*)/uploadImage$",
            r"^/api/v3/store/inventory$",
            r"^/api/v3/store/order$",
            r"^/api/v3/store/order/(?P<orderId>[^/?#]*)$",
            r"^/api/v3/user$",
            r"^/api/v3/user/createWithList$",
            r"^/api/v3/user/login$",
            r"^/api/v3/user/logout$",
            r"^/api/v3/user/(?P<username>[^/?#]*)$"
        ])
        .expect("Unable to create global regex set");
    }
    pub(crate) static ID_PET: usize = 0;
    pub(crate) static ID_PET_FINDBYSTATUS: usize = 1;
    pub(crate) static ID_PET_FINDBYTAGS: usize = 2;
    pub(crate) static ID_PET_PETID: usize = 3;
    lazy_static! {
        pub static ref REGEX_PET_PETID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/pet/(?P<petId>[^/?#]*)$")
                .expect("Unable to create regex for PET_PETID");
    }
    pub(crate) static ID_PET_PETID_UPLOADIMAGE: usize = 4;
    lazy_static! {
        pub static ref REGEX_PET_PETID_UPLOADIMAGE: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/pet/(?P<petId>[^/?#]*)/uploadImage$")
                .expect("Unable to create regex for PET_PETID_UPLOADIMAGE");
    }
    pub(crate) static ID_STORE_INVENTORY: usize = 5;
    pub(crate) static ID_STORE_ORDER: usize = 6;
    pub(crate) static ID_STORE_ORDER_ORDERID: usize = 7;
    lazy_static! {
        pub static ref REGEX_STORE_ORDER_ORDERID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/store/order/(?P<orderId>[^/?#]*)$")
                .expect("Unable to create regex for STORE_ORDER_ORDERID");
    }
    pub(crate) static ID_USER: usize = 8;
    pub(crate) static ID_USER_CREATEWITHLIST: usize = 9;
    pub(crate) static ID_USER_LOGIN: usize = 10;
    pub(crate) static ID_USER_LOGOUT: usize = 11;
    pub(crate) static ID_USER_USERNAME: usize = 12;
    lazy_static! {
        pub static ref REGEX_USER_USERNAME: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/api/v3/user/(?P<username>[^/?#]*)$")
                .expect("Unable to create regex for USER_USERNAME");
    }
}


pub struct MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static
{
    api_impl: T,
    marker: PhantomData<C>,
    validation: bool
}

impl<T, C> MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static
{
    pub fn new(api_impl: T) -> Self {
        MakeService {
            api_impl,
            marker: PhantomData,
            validation: false
        }
    }

    // Turn on/off validation for the service being made.
    #[cfg(feature = "validate")]
    pub fn set_validation(&mut self, validation: bool) {
        self.validation = validation;
    }
}

impl<T, C> Clone for MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Has<Option<Authorization>> + Send + Sync + 'static
{
    fn clone(&self) -> Self {
        Self {
            api_impl: self.api_impl.clone(),
            marker: PhantomData,
            validation: self.validation
        }
    }
}

impl<T, C, Target> hyper::service::Service<Target> for MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static
{
    type Response = Service<T, C>;
    type Error = crate::ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn call(&self, target: Target) -> Self::Future {
        let service = Service::new(self.api_impl.clone(), self.validation);

        future::ok(service)
    }
}

fn method_not_allowed() -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError> {
    Ok(
        Response::builder().status(StatusCode::METHOD_NOT_ALLOWED)
            .body(BoxBody::new(http_body_util::Empty::new()))
            .expect("Unable to create Method Not Allowed response")
    )
}

#[allow(unused_macros)]
#[cfg(not(feature = "validate"))]
macro_rules! run_validation {
    ($parameter:tt, $base_name:tt, $validation:tt) => ();
}

#[allow(unused_macros)]
#[cfg(feature = "validate")]
macro_rules! run_validation {
    ($parameter:tt, $base_name:tt, $validation:tt) => {
        let $parameter = if $validation {
            match $parameter.validate() {
            Ok(()) => $parameter,
            Err(e) => return Ok(Response::builder()
                                    .status(StatusCode::BAD_REQUEST)
                                    .header(CONTENT_TYPE, mime::TEXT_PLAIN.as_ref())
                                    .body(BoxBody::new(format!("Invalid value in body parameter {}: {}", $base_name, e)))
                                    .expect(&format!("Unable to create Bad Request response for invalid value in body parameter {}", $base_name))),
            }
        } else {
            $parameter
        };
    }
}

pub struct Service<T, C> where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static
{
    api_impl: T,
    marker: PhantomData<C>,
    // Enable regex pattern validation of received JSON models
    validation: bool,
}

impl<T, C> Service<T, C> where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static
{
    pub fn new(api_impl: T, validation: bool) -> Self {
        Service {
            api_impl,
            marker: PhantomData,
            validation,
        }
    }
    #[cfg(feature = "validate")]
    pub fn set_validation(&mut self, validation: bool) {
        self.validation = validation
    }

}

impl<T, C> Clone for Service<T, C> where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static
{
    fn clone(&self) -> Self {
        Service {
            api_impl: self.api_impl.clone(),
            marker: self.marker,
            validation: self.validation,
        }
    }
}

#[allow(dead_code)]
fn body_from_string(s: String) -> BoxBody<Bytes, Infallible> {
    BoxBody::new(Full::new(Bytes::from(s)))
}

fn body_from_str(s: &str) -> BoxBody<Bytes, Infallible> {
    BoxBody::new(Full::new(Bytes::copy_from_slice(s.as_bytes())))
}

impl<T, C, ReqBody> hyper::service::Service<(Request<ReqBody>, C)> for Service<T, C> where
    T: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
    type Response = Response<BoxBody<Bytes, Infallible>>;
    type Error = crate::ServiceError;
    type Future = ServiceFuture;

    fn call(&self, req: (Request<ReqBody>, C)) -> Self::Future {
        async fn run<T, C, ReqBody>(
            mut api_impl: T,
            req: (Request<ReqBody>, C),
            validation: bool,
        ) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
        where
            T: Api<C> + Clone + Send + 'static,
            C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
            ReqBody: Body + Send + 'static,
            ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
            ReqBody::Data: Send,
        {
            let (request, context) = req;
            let (parts, body) = request.into_parts();
            let (method, uri, headers) = (parts.method, parts.uri, parts.headers);
            let path = paths::GLOBAL_REGEX_SET.matches(uri.path());

            match method {

            // AddPet - POST /pet
            hyper::Method::POST if path.matched(paths::ID_PET) => {
                handle_add_pet(api_impl, uri, headers, body, context, validation).await
            },

            // FindPetsByStatus - GET /pet/findByStatus
            hyper::Method::GET if path.matched(paths::ID_PET_FINDBYSTATUS) => {
                handle_find_pets_by_status(api_impl, uri, headers, body, context, validation).await
            },

            // FindPetsByTags - GET /pet/findByTags
            hyper::Method::GET if path.matched(paths::ID_PET_FINDBYTAGS) => {
                handle_find_pets_by_tags(api_impl, uri, headers, body, context, validation).await
            },

            // UpdatePet - PUT /pet
            hyper::Method::PUT if path.matched(paths::ID_PET) => {
                handle_update_pet(api_impl, uri, headers, body, context, validation).await
            },

            // DeletePet - DELETE /pet/{petId}
            hyper::Method::DELETE if path.matched(paths::ID_PET_PETID) => {
                handle_delete_pet(api_impl, uri, headers, body, context, validation).await
            },

            // GetPetById - GET /pet/{petId}
            hyper::Method::GET if path.matched(paths::ID_PET_PETID) => {
                handle_get_pet_by_id(api_impl, uri, headers, body, context, validation).await
            },

            // UpdatePetWithForm - POST /pet/{petId}
            hyper::Method::POST if path.matched(paths::ID_PET_PETID) => {
                handle_update_pet_with_form(api_impl, uri, headers, body, context, validation).await
            },

            // UploadFile - POST /pet/{petId}/uploadImage
            hyper::Method::POST if path.matched(paths::ID_PET_PETID_UPLOADIMAGE) => {
                handle_upload_file(api_impl, uri, headers, body, context, validation).await
            },

            // GetInventory - GET /store/inventory
            hyper::Method::GET if path.matched(paths::ID_STORE_INVENTORY) => {
                handle_get_inventory(api_impl, uri, headers, body, context, validation).await
            },

            // PlaceOrder - POST /store/order
            hyper::Method::POST if path.matched(paths::ID_STORE_ORDER) => {
                handle_place_order(api_impl, uri, headers, body, context, validation).await
            },

            // DeleteOrder - DELETE /store/order/{orderId}
            hyper::Method::DELETE if path.matched(paths::ID_STORE_ORDER_ORDERID) => {
                handle_delete_order(api_impl, uri, headers, body, context, validation).await
            },

            // GetOrderById - GET /store/order/{orderId}
            hyper::Method::GET if path.matched(paths::ID_STORE_ORDER_ORDERID) => {
                handle_get_order_by_id(api_impl, uri, headers, body, context, validation).await
            },

            // CreateUser - POST /user
            hyper::Method::POST if path.matched(paths::ID_USER) => {
                handle_create_user(api_impl, uri, headers, body, context, validation).await
            },

            // CreateUsersWithListInput - POST /user/createWithList
            hyper::Method::POST if path.matched(paths::ID_USER_CREATEWITHLIST) => {
                handle_create_users_with_list_input(api_impl, uri, headers, body, context, validation).await
            },

            // LoginUser - GET /user/login
            hyper::Method::GET if path.matched(paths::ID_USER_LOGIN) => {
                handle_login_user(api_impl, uri, headers, body, context, validation).await
            },

            // LogoutUser - GET /user/logout
            hyper::Method::GET if path.matched(paths::ID_USER_LOGOUT) => {
                handle_logout_user(api_impl, uri, headers, body, context, validation).await
            },

            // DeleteUser - DELETE /user/{username}
            hyper::Method::DELETE if path.matched(paths::ID_USER_USERNAME) => {
                handle_delete_user(api_impl, uri, headers, body, context, validation).await
            },

            // GetUserByName - GET /user/{username}
            hyper::Method::GET if path.matched(paths::ID_USER_USERNAME) => {
                handle_get_user_by_name(api_impl, uri, headers, body, context, validation).await
            },

            // UpdateUser - PUT /user/{username}
            hyper::Method::PUT if path.matched(paths::ID_USER_USERNAME) => {
                handle_update_user(api_impl, uri, headers, body, context, validation).await
            },

            _ if path.matched(paths::ID_PET) => method_not_allowed(),
            _ if path.matched(paths::ID_PET_FINDBYSTATUS) => method_not_allowed(),
            _ if path.matched(paths::ID_PET_FINDBYTAGS) => method_not_allowed(),
            _ if path.matched(paths::ID_PET_PETID) => method_not_allowed(),
            _ if path.matched(paths::ID_PET_PETID_UPLOADIMAGE) => method_not_allowed(),
            _ if path.matched(paths::ID_STORE_INVENTORY) => method_not_allowed(),
            _ if path.matched(paths::ID_STORE_ORDER) => method_not_allowed(),
            _ if path.matched(paths::ID_STORE_ORDER_ORDERID) => method_not_allowed(),
            _ if path.matched(paths::ID_USER) => method_not_allowed(),
            _ if path.matched(paths::ID_USER_CREATEWITHLIST) => method_not_allowed(),
            _ if path.matched(paths::ID_USER_LOGIN) => method_not_allowed(),
            _ if path.matched(paths::ID_USER_LOGOUT) => method_not_allowed(),
            _ if path.matched(paths::ID_USER_USERNAME) => method_not_allowed(),
                _ => Ok(Response::builder().status(StatusCode::NOT_FOUND)
                        .body(BoxBody::new(http_body_util::Empty::new()))
                        .expect("Unable to create Not Found response"))
            }
        }
        Box::pin(run(
            self.api_impl.clone(),
            req,
            self.validation
        ))
    }
}

#[allow(unused_variables)]
async fn handle_add_pet<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };

                    // Authorization
                    if let Scopes::Some(ref scopes) = authorization.scopes {
                        let required_scopes: std::collections::BTreeSet<String> = vec![
                            "write:pets".to_string(), // modify pets in your account
                            "read:pets".to_string(), // read your pets
                        ].into_iter().collect();

                        if !required_scopes.is_subset(scopes) {
                            let missing_scopes = required_scopes.difference(scopes);
                            return Ok(Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(BoxBody::new(missing_scopes.fold(
                                    "Insufficient authorization, missing scopes".to_string(),
                                    |s, scope| format!("{s} {scope}"))
                                ))
                                .expect("Unable to create Authentication Insufficient response")
                            );
                        }
                    }
                }

                // Handle body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                let result = http_body_util::BodyExt::collect(body).await.map(|f| f.to_bytes().to_vec());
                match result {
                     Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_pet: Option<models::Pet> = if !body.is_empty() {
                                    let deserializer = &mut serde_xml_rs::de::Deserializer::new_from_reader(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {path}");
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_pet) => param_pet,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(BoxBody::new(format!("Couldn't parse body parameter Pet - doesn't match schema: {e}")))
                                                        .expect("Unable to create Bad Request response for invalid body parameter Pet due to schema")),
                                    }

                                } else {
                                    None
                                };
                                let param_pet = match param_pet {
                                    Some(param_pet) => param_pet,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(BoxBody::new("Missing required body parameter Pet".to_string()))
                                                        .expect("Unable to create Bad Request response for missing body parameter Pet")),
                                };
        #[cfg(not(feature = "validate"))]
                                run_validation!(param_pet, "Pet", validation);


                                let result = api_impl.add_pet(
                                            param_pet,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                AddPetResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                AddPetResponse::InvalidInput
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                AddPetResponse::ValidationException
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");

                                                },
                                                AddPetResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(body_from_string(format!("Unable to read body: {}", e.into())))
                                                .expect("Unable to create Bad Request response due to unable to read body")),
                        }
}

#[allow(unused_variables)]
async fn handle_find_pets_by_status<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };

                    // Authorization
                    if let Scopes::Some(ref scopes) = authorization.scopes {
                        let required_scopes: std::collections::BTreeSet<String> = vec![
                            "write:pets".to_string(), // modify pets in your account
                            "read:pets".to_string(), // read your pets
                        ].into_iter().collect();

                        if !required_scopes.is_subset(scopes) {
                            let missing_scopes = required_scopes.difference(scopes);
                            return Ok(Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(BoxBody::new(missing_scopes.fold(
                                    "Insufficient authorization, missing scopes".to_string(),
                                    |s, scope| format!("{s} {scope}"))
                                ))
                                .expect("Unable to create Authentication Insufficient response")
                            );
                        }
                    }
                }

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                let param_status = query_params.iter().filter(|e| e.0 == "status").map(|e| e.1.clone())
                    .next();
                let param_status = match param_status {
                    Some(param_status) => {
                        let param_status =
                            <models::FindPetsByStatusStatusParameter as std::str::FromStr>::from_str
                                (&param_status);
                        match param_status {
                            Ok(param_status) => Some(param_status),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(body_from_string(format!("Couldn't parse query parameter status - doesn't match schema: {e}")))
                                .expect("Unable to create Bad Request response for invalid query parameter status")),
                        }
                    },
                    None => None,
                };
    #[cfg(not(feature = "validate"))]
                run_validation!(param_status, "status", validation);

                                let result = api_impl.find_pets_by_status(
                                            param_status,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                FindPetsByStatusResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                FindPetsByStatusResponse::InvalidStatusValue
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                FindPetsByStatusResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_find_pets_by_tags<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };

                    // Authorization
                    if let Scopes::Some(ref scopes) = authorization.scopes {
                        let required_scopes: std::collections::BTreeSet<String> = vec![
                            "write:pets".to_string(), // modify pets in your account
                            "read:pets".to_string(), // read your pets
                        ].into_iter().collect();

                        if !required_scopes.is_subset(scopes) {
                            let missing_scopes = required_scopes.difference(scopes);
                            return Ok(Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(BoxBody::new(missing_scopes.fold(
                                    "Insufficient authorization, missing scopes".to_string(),
                                    |s, scope| format!("{s} {scope}"))
                                ))
                                .expect("Unable to create Authentication Insufficient response")
                            );
                        }
                    }
                }

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                let param_tags = query_params.iter().filter(|e| e.0 == "tags").map(|e| e.1.clone())
                    .filter_map(|param_tags| param_tags.parse().ok())
                    .collect::<Vec<_>>();
                let param_tags = if !param_tags.is_empty() {
                    Some(param_tags)
                } else {
                    None
                };

                                let result = api_impl.find_pets_by_tags(
                                            param_tags.as_ref(),
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                FindPetsByTagsResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                FindPetsByTagsResponse::InvalidTagValue
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                FindPetsByTagsResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_update_pet<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };

                    // Authorization
                    if let Scopes::Some(ref scopes) = authorization.scopes {
                        let required_scopes: std::collections::BTreeSet<String> = vec![
                            "write:pets".to_string(), // modify pets in your account
                            "read:pets".to_string(), // read your pets
                        ].into_iter().collect();

                        if !required_scopes.is_subset(scopes) {
                            let missing_scopes = required_scopes.difference(scopes);
                            return Ok(Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(BoxBody::new(missing_scopes.fold(
                                    "Insufficient authorization, missing scopes".to_string(),
                                    |s, scope| format!("{s} {scope}"))
                                ))
                                .expect("Unable to create Authentication Insufficient response")
                            );
                        }
                    }
                }

                // Handle body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                let result = http_body_util::BodyExt::collect(body).await.map(|f| f.to_bytes().to_vec());
                match result {
                     Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_pet: Option<models::Pet> = if !body.is_empty() {
                                    let deserializer = &mut serde_xml_rs::de::Deserializer::new_from_reader(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {path}");
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_pet) => param_pet,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(BoxBody::new(format!("Couldn't parse body parameter Pet - doesn't match schema: {e}")))
                                                        .expect("Unable to create Bad Request response for invalid body parameter Pet due to schema")),
                                    }

                                } else {
                                    None
                                };
                                let param_pet = match param_pet {
                                    Some(param_pet) => param_pet,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(BoxBody::new("Missing required body parameter Pet".to_string()))
                                                        .expect("Unable to create Bad Request response for missing body parameter Pet")),
                                };
        #[cfg(not(feature = "validate"))]
                                run_validation!(param_pet, "Pet", validation);


                                let result = api_impl.update_pet(
                                            param_pet,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                UpdatePetResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                UpdatePetResponse::InvalidIDSupplied
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                UpdatePetResponse::PetNotFound
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");

                                                },
                                                UpdatePetResponse::ValidationException
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");

                                                },
                                                UpdatePetResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(body_from_string(format!("Unable to read body: {}", e.into())))
                                                .expect("Unable to create Bad Request response due to unable to read body")),
                        }
}

#[allow(unused_variables)]
async fn handle_delete_pet<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };

                    // Authorization
                    if let Scopes::Some(ref scopes) = authorization.scopes {
                        let required_scopes: std::collections::BTreeSet<String> = vec![
                            "write:pets".to_string(), // modify pets in your account
                            "read:pets".to_string(), // read your pets
                        ].into_iter().collect();

                        if !required_scopes.is_subset(scopes) {
                            let missing_scopes = required_scopes.difference(scopes);
                            return Ok(Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(BoxBody::new(missing_scopes.fold(
                                    "Insufficient authorization, missing scopes".to_string(),
                                    |s, scope| format!("{s} {scope}"))
                                ))
                                .expect("Unable to create Authentication Insufficient response")
                            );
                        }
                    }
                }

                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_PET_PETID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE PET_PETID in set but failed match against \"{}\"", path, paths::REGEX_PET_PETID.as_str())
                    );

                let param_pet_id = match percent_encoding::percent_decode(path_params["petId"].as_bytes()).decode_utf8() {
                    Ok(param_pet_id) => match param_pet_id.parse::<i64>() {
                        Ok(param_pet_id) => param_pet_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter petId: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["petId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                // Header parameters
                let param_api_key = headers.get(HeaderName::from_static("api_key"));

                let param_api_key = match param_api_key {
                    Some(v) => match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                        Ok(result) =>
                            Some(result.0),
                        Err(err) => {
                            return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Invalid header api_key - {err}")))
                                        .expect("Unable to create Bad Request response for invalid header api_key"));

                        },
                    },
                    None => {
                        None
                    }
                };

                                let result = api_impl.delete_pet(
                                            param_pet_id,
                                            param_api_key,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                DeletePetResponse::SuccessfulOperation
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");

                                                },
                                                DeletePetResponse::InvalidPetValue
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                DeletePetResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_get_pet_by_id<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };

                    // Authorization
                    if let Scopes::Some(ref scopes) = authorization.scopes {
                        let required_scopes: std::collections::BTreeSet<String> = vec![
                            "write:pets".to_string(), // modify pets in your account
                            "read:pets".to_string(), // read your pets
                        ].into_iter().collect();

                        if !required_scopes.is_subset(scopes) {
                            let missing_scopes = required_scopes.difference(scopes);
                            return Ok(Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(BoxBody::new(missing_scopes.fold(
                                    "Insufficient authorization, missing scopes".to_string(),
                                    |s, scope| format!("{s} {scope}"))
                                ))
                                .expect("Unable to create Authentication Insufficient response")
                            );
                        }
                    }
                }

                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_PET_PETID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE PET_PETID in set but failed match against \"{}\"", path, paths::REGEX_PET_PETID.as_str())
                    );

                let param_pet_id = match percent_encoding::percent_decode(path_params["petId"].as_bytes()).decode_utf8() {
                    Ok(param_pet_id) => match param_pet_id.parse::<i64>() {
                        Ok(param_pet_id) => param_pet_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter petId: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["petId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                                let result = api_impl.get_pet_by_id(
                                            param_pet_id,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                GetPetByIdResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                GetPetByIdResponse::InvalidIDSupplied
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                GetPetByIdResponse::PetNotFound
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");

                                                },
                                                GetPetByIdResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_update_pet_with_form<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };

                    // Authorization
                    if let Scopes::Some(ref scopes) = authorization.scopes {
                        let required_scopes: std::collections::BTreeSet<String> = vec![
                            "write:pets".to_string(), // modify pets in your account
                            "read:pets".to_string(), // read your pets
                        ].into_iter().collect();

                        if !required_scopes.is_subset(scopes) {
                            let missing_scopes = required_scopes.difference(scopes);
                            return Ok(Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(BoxBody::new(missing_scopes.fold(
                                    "Insufficient authorization, missing scopes".to_string(),
                                    |s, scope| format!("{s} {scope}"))
                                ))
                                .expect("Unable to create Authentication Insufficient response")
                            );
                        }
                    }
                }

                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_PET_PETID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE PET_PETID in set but failed match against \"{}\"", path, paths::REGEX_PET_PETID.as_str())
                    );

                let param_pet_id = match percent_encoding::percent_decode(path_params["petId"].as_bytes()).decode_utf8() {
                    Ok(param_pet_id) => match param_pet_id.parse::<i64>() {
                        Ok(param_pet_id) => param_pet_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter petId: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["petId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                let param_name = query_params.iter().filter(|e| e.0 == "name").map(|e| e.1.clone())
                    .next();
                let param_name = match param_name {
                    Some(param_name) => {
                        let param_name =
                            <String as std::str::FromStr>::from_str
                                (&param_name);
                        match param_name {
                            Ok(param_name) => Some(param_name),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(body_from_string(format!("Couldn't parse query parameter name - doesn't match schema: {e}")))
                                .expect("Unable to create Bad Request response for invalid query parameter name")),
                        }
                    },
                    None => None,
                };
                let param_status = query_params.iter().filter(|e| e.0 == "status").map(|e| e.1.clone())
                    .next();
                let param_status = match param_status {
                    Some(param_status) => {
                        let param_status =
                            <String as std::str::FromStr>::from_str
                                (&param_status);
                        match param_status {
                            Ok(param_status) => Some(param_status),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(body_from_string(format!("Couldn't parse query parameter status - doesn't match schema: {e}")))
                                .expect("Unable to create Bad Request response for invalid query parameter status")),
                        }
                    },
                    None => None,
                };

                                let result = api_impl.update_pet_with_form(
                                            param_pet_id,
                                            param_name,
                                            param_status,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                UpdatePetWithFormResponse::SuccessfullyUpdated
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");

                                                },
                                                UpdatePetWithFormResponse::InvalidInput
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                UpdatePetWithFormResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_upload_file<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };

                    // Authorization
                    if let Scopes::Some(ref scopes) = authorization.scopes {
                        let required_scopes: std::collections::BTreeSet<String> = vec![
                            "write:pets".to_string(), // modify pets in your account
                            "read:pets".to_string(), // read your pets
                        ].into_iter().collect();

                        if !required_scopes.is_subset(scopes) {
                            let missing_scopes = required_scopes.difference(scopes);
                            return Ok(Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(BoxBody::new(missing_scopes.fold(
                                    "Insufficient authorization, missing scopes".to_string(),
                                    |s, scope| format!("{s} {scope}"))
                                ))
                                .expect("Unable to create Authentication Insufficient response")
                            );
                        }
                    }
                }

                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_PET_PETID_UPLOADIMAGE
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE PET_PETID_UPLOADIMAGE in set but failed match against \"{}\"", path, paths::REGEX_PET_PETID_UPLOADIMAGE.as_str())
                    );

                let param_pet_id = match percent_encoding::percent_decode(path_params["petId"].as_bytes()).decode_utf8() {
                    Ok(param_pet_id) => match param_pet_id.parse::<i64>() {
                        Ok(param_pet_id) => param_pet_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter petId: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["petId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                let param_additional_metadata = query_params.iter().filter(|e| e.0 == "additionalMetadata").map(|e| e.1.clone())
                    .next();
                let param_additional_metadata = match param_additional_metadata {
                    Some(param_additional_metadata) => {
                        let param_additional_metadata =
                            <String as std::str::FromStr>::from_str
                                (&param_additional_metadata);
                        match param_additional_metadata {
                            Ok(param_additional_metadata) => Some(param_additional_metadata),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(body_from_string(format!("Couldn't parse query parameter additionalMetadata - doesn't match schema: {e}")))
                                .expect("Unable to create Bad Request response for invalid query parameter additionalMetadata")),
                        }
                    },
                    None => None,
                };

                // Handle body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                let result = http_body_util::BodyExt::collect(body).await.map(|f| f.to_bytes().to_vec());
                match result {
                     Ok(body) => {
                                let param_body: Option<swagger::ByteArray> = if !body.is_empty() {
                                    Some(swagger::ByteArray(body.to_vec()))
                                } else {
                                    None
                                };


                                let result = api_impl.upload_file(
                                            param_pet_id,
                                            param_additional_metadata,
                                            param_body,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                UploadFileResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                UploadFileResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(body_from_string(format!("Unable to read body: {}", e.into())))
                                                .expect("Unable to create Bad Request response due to unable to read body")),
                        }
}

#[allow(unused_variables)]
async fn handle_get_inventory<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                {
                    let authorization = match *(&context as &dyn Has<Option<Authorization>>).get() {
                        Some(ref authorization) => authorization,
                        None => return Ok(Response::builder()
                                                .status(StatusCode::FORBIDDEN)
                                                .body(body_from_str("Unauthenticated"))
                                                .expect("Unable to create Authentication Forbidden response")),
                    };
                }

                                let result = api_impl.get_inventory(
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                GetInventoryResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                GetInventoryResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_place_order<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Handle body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                let result = http_body_util::BodyExt::collect(body).await.map(|f| f.to_bytes().to_vec());
                match result {
                     Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_order: Option<models::Order> = if !body.is_empty() {
                                    let deserializer = &mut serde_xml_rs::de::Deserializer::new_from_reader(&*body);
                                    serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {path}");
                                        unused_elements.push(path.to_string());
                                    }).unwrap_or_default()

                                } else {
                                    None
                                };
        #[cfg(not(feature = "validate"))]
                                run_validation!(param_order, "Order", validation);


                                let result = api_impl.place_order(
                                            param_order,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                PlaceOrderResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                PlaceOrderResponse::InvalidInput
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                PlaceOrderResponse::ValidationException
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(422).expect("Unable to turn 422 into a StatusCode");

                                                },
                                                PlaceOrderResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(body_from_string(format!("Unable to read body: {}", e.into())))
                                                .expect("Unable to create Bad Request response due to unable to read body")),
                        }
}

#[allow(unused_variables)]
async fn handle_delete_order<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_STORE_ORDER_ORDERID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE STORE_ORDER_ORDERID in set but failed match against \"{}\"", path, paths::REGEX_STORE_ORDER_ORDERID.as_str())
                    );

                let param_order_id = match percent_encoding::percent_decode(path_params["orderId"].as_bytes()).decode_utf8() {
                    Ok(param_order_id) => match param_order_id.parse::<i64>() {
                        Ok(param_order_id) => param_order_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter orderId: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["orderId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                                let result = api_impl.delete_order(
                                            param_order_id,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                DeleteOrderResponse::SuccessfulOperation
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");

                                                },
                                                DeleteOrderResponse::InvalidIDSupplied
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                DeleteOrderResponse::OrderNotFound
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");

                                                },
                                                DeleteOrderResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_get_order_by_id<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_STORE_ORDER_ORDERID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE STORE_ORDER_ORDERID in set but failed match against \"{}\"", path, paths::REGEX_STORE_ORDER_ORDERID.as_str())
                    );

                let param_order_id = match percent_encoding::percent_decode(path_params["orderId"].as_bytes()).decode_utf8() {
                    Ok(param_order_id) => match param_order_id.parse::<i64>() {
                        Ok(param_order_id) => param_order_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter orderId: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["orderId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                                let result = api_impl.get_order_by_id(
                                            param_order_id,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                GetOrderByIdResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                GetOrderByIdResponse::InvalidIDSupplied
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                GetOrderByIdResponse::OrderNotFound
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");

                                                },
                                                GetOrderByIdResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_create_user<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Handle body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                let result = http_body_util::BodyExt::collect(body).await.map(|f| f.to_bytes().to_vec());
                match result {
                     Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_user: Option<models::User> = if !body.is_empty() {
                                    let deserializer = &mut serde_xml_rs::de::Deserializer::new_from_reader(&*body);
                                    serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {path}");
                                        unused_elements.push(path.to_string());
                                    }).unwrap_or_default()

                                } else {
                                    None
                                };
        #[cfg(not(feature = "validate"))]
                                run_validation!(param_user, "User", validation);


                                let result = api_impl.create_user(
                                            param_user,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                CreateUserResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                CreateUserResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(body_from_string(format!("Unable to read body: {}", e.into())))
                                                .expect("Unable to create Bad Request response due to unable to read body")),
                        }
}

#[allow(unused_variables)]
async fn handle_create_users_with_list_input<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Handle body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                let result = http_body_util::BodyExt::collect(body).await.map(|f| f.to_bytes().to_vec());
                match result {
                     Ok(body) => {
                                let mut unused_elements : Vec<String> = vec![];
                                let param_user: Option<Vec<models::User>> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&body);
                                    serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {path}");
                                        unused_elements.push(path.to_string());
                                    }).unwrap_or_default()

                                } else {
                                    None
                                };
        #[cfg(not(feature = "validate"))]
                                run_validation!(param_user, "User", validation);


                                let result = api_impl.create_users_with_list_input(
                                            param_user.as_ref(),
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {unused_elements:?}").as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }
                                        match result {
                                            Ok(rsp) => match rsp {
                                                CreateUsersWithListInputResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                CreateUsersWithListInputResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(body_from_string(format!("Unable to read body: {}", e.into())))
                                                .expect("Unable to create Bad Request response due to unable to read body")),
                        }
}

#[allow(unused_variables)]
async fn handle_login_user<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                let query_params = form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect::<Vec<_>>();
                let param_username = query_params.iter().filter(|e| e.0 == "username").map(|e| e.1.clone())
                    .next();
                let param_username = match param_username {
                    Some(param_username) => {
                        let param_username =
                            <String as std::str::FromStr>::from_str
                                (&param_username);
                        match param_username {
                            Ok(param_username) => Some(param_username),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(body_from_string(format!("Couldn't parse query parameter username - doesn't match schema: {e}")))
                                .expect("Unable to create Bad Request response for invalid query parameter username")),
                        }
                    },
                    None => None,
                };
                let param_password = query_params.iter().filter(|e| e.0 == "password").map(|e| e.1.clone())
                    .next();
                let param_password = match param_password {
                    Some(param_password) => {
                        let param_password =
                            <String as std::str::FromStr>::from_str
                                (&param_password);
                        match param_password {
                            Ok(param_password) => Some(param_password),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(body_from_string(format!("Couldn't parse query parameter password - doesn't match schema: {e}")))
                                .expect("Unable to create Bad Request response for invalid query parameter password")),
                        }
                    },
                    None => None,
                };

                                let result = api_impl.login_user(
                                            param_username,
                                            param_password,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                LoginUserResponse::SuccessfulOperation
                                                    {
                                                        body,
                                                        x_rate_limit,
                                                        x_expires_after
                                                    }
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");

                                                    if let Some(x_rate_limit) = x_rate_limit {
                                                    let x_rate_limit = match header::IntoHeaderValue(x_rate_limit).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(body_from_string(format!("An internal server error occurred handling x_rate_limit header - {e}")))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("x-rate-limit"),
                                                        x_rate_limit
                                                    );
                                                    }

                                                    if let Some(x_expires_after) = x_expires_after {
                                                    let x_expires_after = match header::IntoHeaderValue(x_expires_after).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(body_from_string(format!("An internal server error occurred handling x_expires_after header - {e}")))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("x-expires-after"),
                                                        x_expires_after
                                                    );
                                                    }
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // String body returned directly
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                LoginUserResponse::InvalidUsername
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                LoginUserResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_logout_user<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                                let result = api_impl.logout_user(
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                LogoutUserResponse::SuccessfulOperation
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");

                                                },
                                                LogoutUserResponse::SuccessfulOperation_2
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_delete_user<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_USER_USERNAME
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE USER_USERNAME in set but failed match against \"{}\"", path, paths::REGEX_USER_USERNAME.as_str())
                    );

                let param_username = match percent_encoding::percent_decode(path_params["username"].as_bytes()).decode_utf8() {
                    Ok(param_username) => match param_username.parse::<String>() {
                        Ok(param_username) => param_username,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter username: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["username"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                                let result = api_impl.delete_user(
                                            param_username,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                DeleteUserResponse::SuccessfulOperation
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");

                                                },
                                                DeleteUserResponse::InvalidUsernameSupplied
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                DeleteUserResponse::UserNotFound
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");

                                                },
                                                DeleteUserResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_get_user_by_name<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_USER_USERNAME
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE USER_USERNAME in set but failed match against \"{}\"", path, paths::REGEX_USER_USERNAME.as_str())
                    );

                let param_username = match percent_encoding::percent_decode(path_params["username"].as_bytes()).decode_utf8() {
                    Ok(param_username) => match param_username.parse::<String>() {
                        Ok(param_username) => param_username,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter username: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["username"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                                let result = api_impl.get_user_by_name(
                                            param_username,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                GetUserByNameResponse::SuccessfulOperation
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                                GetUserByNameResponse::InvalidUsernameSupplied
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");

                                                },
                                                GetUserByNameResponse::UserNotFound
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");

                                                },
                                                GetUserByNameResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
}

#[allow(unused_variables)]
async fn handle_update_user<T, C, ReqBody>(
    mut api_impl: T,
    uri: hyper::Uri,
    headers: HeaderMap,
    body: ReqBody,
    context: C,
    validation: bool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, crate::ServiceError>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString>  + Send + Sync + 'static,
    ReqBody: Body + Send + 'static,
    ReqBody::Error: Into<Box<dyn Error + Send + Sync>> + Send,
    ReqBody::Data: Send,
{
                // Path parameters
                let path: &str = uri.path();
                let path_params =
                    paths::REGEX_USER_USERNAME
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE USER_USERNAME in set but failed match against \"{}\"", path, paths::REGEX_USER_USERNAME.as_str())
                    );

                let param_username = match percent_encoding::percent_decode(path_params["username"].as_bytes()).decode_utf8() {
                    Ok(param_username) => match param_username.parse::<String>() {
                        Ok(param_username) => param_username,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't parse path parameter username: {e}")))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(body_from_string(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["username"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                // Handle body parameters (note that non-required body parameters will ignore garbage
                // values, rather than causing a 400 response). Produce warning header and logs for
                // any unused fields.
                let result = http_body_util::BodyExt::collect(body).await.map(|f| f.to_bytes().to_vec());
                match result {
                     Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_user: Option<models::User> = if !body.is_empty() {
                                    let deserializer = &mut serde_xml_rs::de::Deserializer::new_from_reader(&*body);
                                    serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {path}");
                                        unused_elements.push(path.to_string());
                                    }).unwrap_or_default()

                                } else {
                                    None
                                };
        #[cfg(not(feature = "validate"))]
                                run_validation!(param_user, "User", validation);


                                let result = api_impl.update_user(
                                            param_username,
                                            param_user,
                                        &context
                                    ).await;
                                let mut response = Response::new(BoxBody::new(http_body_util::Empty::new()));
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                UpdateUserResponse::SuccessfulOperation
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");

                                                },
                                                UpdateUserResponse::UnexpectedError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_static("application/json"));
                                                    // JSON Body
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = body_from_string(body);

                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = body_from_str("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(body_from_string(format!("Unable to read body: {}", e.into())))
                                                .expect("Unable to create Bad Request response due to unable to read body")),
                        }
}

/// Request parser for `Api`.
pub struct ApiRequestParser;
impl<T> RequestParser<T> for ApiRequestParser {
    fn parse_operation_id(request: &Request<T>) -> Option<&'static str> {
        let path = paths::GLOBAL_REGEX_SET.matches(request.uri().path());
        match *request.method() {
            // AddPet - POST /pet
            hyper::Method::POST if path.matched(paths::ID_PET) => Some("AddPet"),
            // FindPetsByStatus - GET /pet/findByStatus
            hyper::Method::GET if path.matched(paths::ID_PET_FINDBYSTATUS) => Some("FindPetsByStatus"),
            // FindPetsByTags - GET /pet/findByTags
            hyper::Method::GET if path.matched(paths::ID_PET_FINDBYTAGS) => Some("FindPetsByTags"),
            // UpdatePet - PUT /pet
            hyper::Method::PUT if path.matched(paths::ID_PET) => Some("UpdatePet"),
            // DeletePet - DELETE /pet/{petId}
            hyper::Method::DELETE if path.matched(paths::ID_PET_PETID) => Some("DeletePet"),
            // GetPetById - GET /pet/{petId}
            hyper::Method::GET if path.matched(paths::ID_PET_PETID) => Some("GetPetById"),
            // UpdatePetWithForm - POST /pet/{petId}
            hyper::Method::POST if path.matched(paths::ID_PET_PETID) => Some("UpdatePetWithForm"),
            // UploadFile - POST /pet/{petId}/uploadImage
            hyper::Method::POST if path.matched(paths::ID_PET_PETID_UPLOADIMAGE) => Some("UploadFile"),
            // GetInventory - GET /store/inventory
            hyper::Method::GET if path.matched(paths::ID_STORE_INVENTORY) => Some("GetInventory"),
            // PlaceOrder - POST /store/order
            hyper::Method::POST if path.matched(paths::ID_STORE_ORDER) => Some("PlaceOrder"),
            // DeleteOrder - DELETE /store/order/{orderId}
            hyper::Method::DELETE if path.matched(paths::ID_STORE_ORDER_ORDERID) => Some("DeleteOrder"),
            // GetOrderById - GET /store/order/{orderId}
            hyper::Method::GET if path.matched(paths::ID_STORE_ORDER_ORDERID) => Some("GetOrderById"),
            // CreateUser - POST /user
            hyper::Method::POST if path.matched(paths::ID_USER) => Some("CreateUser"),
            // CreateUsersWithListInput - POST /user/createWithList
            hyper::Method::POST if path.matched(paths::ID_USER_CREATEWITHLIST) => Some("CreateUsersWithListInput"),
            // LoginUser - GET /user/login
            hyper::Method::GET if path.matched(paths::ID_USER_LOGIN) => Some("LoginUser"),
            // LogoutUser - GET /user/logout
            hyper::Method::GET if path.matched(paths::ID_USER_LOGOUT) => Some("LogoutUser"),
            // DeleteUser - DELETE /user/{username}
            hyper::Method::DELETE if path.matched(paths::ID_USER_USERNAME) => Some("DeleteUser"),
            // GetUserByName - GET /user/{username}
            hyper::Method::GET if path.matched(paths::ID_USER_USERNAME) => Some("GetUserByName"),
            // UpdateUser - PUT /user/{username}
            hyper::Method::PUT if path.matched(paths::ID_USER_USERNAME) => Some("UpdateUser"),
            _ => None,
        }
    }
}

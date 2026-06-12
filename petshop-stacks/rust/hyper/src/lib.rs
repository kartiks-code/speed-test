#![allow(missing_docs, trivial_casts, unused_variables, unused_mut, unused_imports, unused_extern_crates, unused_attributes, non_camel_case_types)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::disallowed_names)]

use async_trait::async_trait;
use futures::Stream;
#[cfg(feature = "mock")]
use mockall::automock;
use std::error::Error;
use std::collections::BTreeSet;
use std::task::{Poll, Context};
use swagger::{ApiError, ContextWrapper, auth::Authorization};
use serde::{Serialize, Deserialize};

#[cfg(any(feature = "client", feature = "server"))]
type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &str = "/api/v3";
pub const API_VERSION: &str = "1.0.12";

mod auth;
pub use auth::{AuthenticationApi, Claims};


#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum AddPetResponse {
    /// Successful operation
    SuccessfulOperation
    (models::Pet)
    ,
    /// Invalid input
    InvalidInput
    ,
    /// Validation exception
    ValidationException
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum FindPetsByStatusResponse {
    /// successful operation
    SuccessfulOperation
    (Vec<models::Pet>)
    ,
    /// Invalid status value
    InvalidStatusValue
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum FindPetsByTagsResponse {
    /// successful operation
    SuccessfulOperation
    (Vec<models::Pet>)
    ,
    /// Invalid tag value
    InvalidTagValue
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum UpdatePetResponse {
    /// Successful operation
    SuccessfulOperation
    (models::Pet)
    ,
    /// Invalid ID supplied
    InvalidIDSupplied
    ,
    /// Pet not found
    PetNotFound
    ,
    /// Validation exception
    ValidationException
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum DeletePetResponse {
    /// successful operation
    SuccessfulOperation
    ,
    /// Invalid pet value
    InvalidPetValue
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum GetPetByIdResponse {
    /// successful operation
    SuccessfulOperation
    (models::Pet)
    ,
    /// Invalid ID supplied
    InvalidIDSupplied
    ,
    /// Pet not found
    PetNotFound
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum UpdatePetWithFormResponse {
    /// successfully updated
    SuccessfullyUpdated
    ,
    /// Invalid input
    InvalidInput
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum UploadFileResponse {
    /// successful operation
    SuccessfulOperation
    (models::ApiResponse)
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum GetInventoryResponse {
    /// successful operation
    SuccessfulOperation
    (std::collections::HashMap<String, i32>)
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum PlaceOrderResponse {
    /// successful operation
    SuccessfulOperation
    (models::Order)
    ,
    /// Invalid input
    InvalidInput
    ,
    /// Validation exception
    ValidationException
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum DeleteOrderResponse {
    /// successful operation
    SuccessfulOperation
    ,
    /// Invalid ID supplied
    InvalidIDSupplied
    ,
    /// Order not found
    OrderNotFound
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum GetOrderByIdResponse {
    /// successful operation
    SuccessfulOperation
    (models::Order)
    ,
    /// Invalid ID supplied
    InvalidIDSupplied
    ,
    /// Order not found
    OrderNotFound
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum CreateUserResponse {
    /// successful operation
    SuccessfulOperation
    (models::User)
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum CreateUsersWithListInputResponse {
    /// Successful operation
    SuccessfulOperation
    (models::User)
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum LoginUserResponse {
    /// successful operation
    SuccessfulOperation
    {
        body: String,
        x_rate_limit:
        Option<
        i32
        >
        ,
        x_expires_after:
        Option<
        chrono::DateTime::<chrono::Utc>
        >
    }
    ,
    /// Invalid username/password supplied
    InvalidUsername
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum LogoutUserResponse {
    /// successful operation
    SuccessfulOperation
    ,
    /// successful operation
    SuccessfulOperation_2
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum DeleteUserResponse {
    /// successful operation
    SuccessfulOperation
    ,
    /// Invalid username supplied
    InvalidUsernameSupplied
    ,
    /// User not found
    UserNotFound
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum GetUserByNameResponse {
    /// successful operation
    SuccessfulOperation
    (models::User)
    ,
    /// Invalid username supplied
    InvalidUsernameSupplied
    ,
    /// User not found
    UserNotFound
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum UpdateUserResponse {
    /// successful operation
    SuccessfulOperation
    ,
    /// Unexpected error
    UnexpectedError
    (models::Error)
}

/// API
#[cfg_attr(feature = "mock", automock)]
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait Api<C: Send + Sync> {
    /// Add a new pet to the store.
    async fn add_pet(
        &self,
        pet: models::Pet,
        context: &C) -> Result<AddPetResponse, ApiError>;

    /// Finds Pets by status.
    async fn find_pets_by_status(
        &self,
        status: Option<models::FindPetsByStatusStatusParameter>,
        context: &C) -> Result<FindPetsByStatusResponse, ApiError>;

    /// Finds Pets by tags.
    async fn find_pets_by_tags<'a>(
        &self,
        tags: Option<&'a Vec<String>>,
        context: &C) -> Result<FindPetsByTagsResponse, ApiError>;

    /// Update an existing pet.
    async fn update_pet(
        &self,
        pet: models::Pet,
        context: &C) -> Result<UpdatePetResponse, ApiError>;

    /// Deletes a pet.
    async fn delete_pet(
        &self,
        pet_id: i64,
        api_key: Option<String>,
        context: &C) -> Result<DeletePetResponse, ApiError>;

    /// Find pet by identifier.
    async fn get_pet_by_id(
        &self,
        pet_id: i64,
        context: &C) -> Result<GetPetByIdResponse, ApiError>;

    /// Updates a pet in the store with form data.
    async fn update_pet_with_form(
        &self,
        pet_id: i64,
        name: Option<String>,
        status: Option<String>,
        context: &C) -> Result<UpdatePetWithFormResponse, ApiError>;

    /// Uploads an image.
    async fn upload_file(
        &self,
        pet_id: i64,
        additional_metadata: Option<String>,
        body: Option<swagger::ByteArray>,
        context: &C) -> Result<UploadFileResponse, ApiError>;

    /// Returns pet inventories by status.
    async fn get_inventory(
        &self,
        context: &C) -> Result<GetInventoryResponse, ApiError>;

    /// Place an order for a pet.
    async fn place_order(
        &self,
        order: Option<models::Order>,
        context: &C) -> Result<PlaceOrderResponse, ApiError>;

    /// Delete purchase order by identifier.
    async fn delete_order(
        &self,
        order_id: i64,
        context: &C) -> Result<DeleteOrderResponse, ApiError>;

    /// Find purchase order by identifier.
    async fn get_order_by_id(
        &self,
        order_id: i64,
        context: &C) -> Result<GetOrderByIdResponse, ApiError>;

    /// Create user.
    async fn create_user(
        &self,
        user: Option<models::User>,
        context: &C) -> Result<CreateUserResponse, ApiError>;

    /// Creates list of users with given input array.
    async fn create_users_with_list_input<'a>(
        &self,
        user: Option<&'a Vec<models::User>>,
        context: &C) -> Result<CreateUsersWithListInputResponse, ApiError>;

    /// Logs user into the system.
    async fn login_user(
        &self,
        username: Option<String>,
        password: Option<String>,
        context: &C) -> Result<LoginUserResponse, ApiError>;

    /// Logs out current logged in user session.
    async fn logout_user(
        &self,
        context: &C) -> Result<LogoutUserResponse, ApiError>;

    /// Delete user.
    async fn delete_user(
        &self,
        username: String,
        context: &C) -> Result<DeleteUserResponse, ApiError>;

    /// Get user by user name.
    async fn get_user_by_name(
        &self,
        username: String,
        context: &C) -> Result<GetUserByNameResponse, ApiError>;

    /// Update user.
    async fn update_user(
        &self,
        username: String,
        user: Option<models::User>,
        context: &C) -> Result<UpdateUserResponse, ApiError>;

}

/// API where `Context` isn't passed on every API call
#[cfg_attr(feature = "mock", automock)]
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait ApiNoContext<C: Send + Sync> {
    // The std::task::Context struct houses a reference to std::task::Waker with the lifetime <'a>.
    // Adding an anonymous lifetime `'a` to allow mockall to create a mock object with the right lifetimes.
    // This is needed because the compiler is unable to determine the lifetimes on F's trait bound
    // where F is the closure created by mockall. We use higher-rank trait bounds here to get around this.

    fn context(&self) -> &C;

    /// Add a new pet to the store.
    async fn add_pet(
        &self,
        pet: models::Pet,
        ) -> Result<AddPetResponse, ApiError>;

    /// Finds Pets by status.
    async fn find_pets_by_status(
        &self,
        status: Option<models::FindPetsByStatusStatusParameter>,
        ) -> Result<FindPetsByStatusResponse, ApiError>;

    /// Finds Pets by tags.
    async fn find_pets_by_tags<'a>(
        &self,
        tags: Option<&'a Vec<String>>,
        ) -> Result<FindPetsByTagsResponse, ApiError>;

    /// Update an existing pet.
    async fn update_pet(
        &self,
        pet: models::Pet,
        ) -> Result<UpdatePetResponse, ApiError>;

    /// Deletes a pet.
    async fn delete_pet(
        &self,
        pet_id: i64,
        api_key: Option<String>,
        ) -> Result<DeletePetResponse, ApiError>;

    /// Find pet by identifier.
    async fn get_pet_by_id(
        &self,
        pet_id: i64,
        ) -> Result<GetPetByIdResponse, ApiError>;

    /// Updates a pet in the store with form data.
    async fn update_pet_with_form(
        &self,
        pet_id: i64,
        name: Option<String>,
        status: Option<String>,
        ) -> Result<UpdatePetWithFormResponse, ApiError>;

    /// Uploads an image.
    async fn upload_file(
        &self,
        pet_id: i64,
        additional_metadata: Option<String>,
        body: Option<swagger::ByteArray>,
        ) -> Result<UploadFileResponse, ApiError>;

    /// Returns pet inventories by status.
    async fn get_inventory(
        &self,
        ) -> Result<GetInventoryResponse, ApiError>;

    /// Place an order for a pet.
    async fn place_order(
        &self,
        order: Option<models::Order>,
        ) -> Result<PlaceOrderResponse, ApiError>;

    /// Delete purchase order by identifier.
    async fn delete_order(
        &self,
        order_id: i64,
        ) -> Result<DeleteOrderResponse, ApiError>;

    /// Find purchase order by identifier.
    async fn get_order_by_id(
        &self,
        order_id: i64,
        ) -> Result<GetOrderByIdResponse, ApiError>;

    /// Create user.
    async fn create_user(
        &self,
        user: Option<models::User>,
        ) -> Result<CreateUserResponse, ApiError>;

    /// Creates list of users with given input array.
    async fn create_users_with_list_input<'a>(
        &self,
        user: Option<&'a Vec<models::User>>,
        ) -> Result<CreateUsersWithListInputResponse, ApiError>;

    /// Logs user into the system.
    async fn login_user(
        &self,
        username: Option<String>,
        password: Option<String>,
        ) -> Result<LoginUserResponse, ApiError>;

    /// Logs out current logged in user session.
    async fn logout_user(
        &self,
        ) -> Result<LogoutUserResponse, ApiError>;

    /// Delete user.
    async fn delete_user(
        &self,
        username: String,
        ) -> Result<DeleteUserResponse, ApiError>;

    /// Get user by user name.
    async fn get_user_by_name(
        &self,
        username: String,
        ) -> Result<GetUserByNameResponse, ApiError>;

    /// Update user.
    async fn update_user(
        &self,
        username: String,
        user: Option<models::User>,
        ) -> Result<UpdateUserResponse, ApiError>;

}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync> where Self: Sized
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
         ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    /// Add a new pet to the store.
    async fn add_pet(
        &self,
        pet: models::Pet,
        ) -> Result<AddPetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().add_pet(pet, &context).await
    }

    /// Finds Pets by status.
    async fn find_pets_by_status(
        &self,
        status: Option<models::FindPetsByStatusStatusParameter>,
        ) -> Result<FindPetsByStatusResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().find_pets_by_status(status, &context).await
    }

    /// Finds Pets by tags.
    async fn find_pets_by_tags<'a>(
        &self,
        tags: Option<&'a Vec<String>>,
        ) -> Result<FindPetsByTagsResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().find_pets_by_tags(tags, &context).await
    }

    /// Update an existing pet.
    async fn update_pet(
        &self,
        pet: models::Pet,
        ) -> Result<UpdatePetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().update_pet(pet, &context).await
    }

    /// Deletes a pet.
    async fn delete_pet(
        &self,
        pet_id: i64,
        api_key: Option<String>,
        ) -> Result<DeletePetResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().delete_pet(pet_id, api_key, &context).await
    }

    /// Find pet by identifier.
    async fn get_pet_by_id(
        &self,
        pet_id: i64,
        ) -> Result<GetPetByIdResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().get_pet_by_id(pet_id, &context).await
    }

    /// Updates a pet in the store with form data.
    async fn update_pet_with_form(
        &self,
        pet_id: i64,
        name: Option<String>,
        status: Option<String>,
        ) -> Result<UpdatePetWithFormResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().update_pet_with_form(pet_id, name, status, &context).await
    }

    /// Uploads an image.
    async fn upload_file(
        &self,
        pet_id: i64,
        additional_metadata: Option<String>,
        body: Option<swagger::ByteArray>,
        ) -> Result<UploadFileResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().upload_file(pet_id, additional_metadata, body, &context).await
    }

    /// Returns pet inventories by status.
    async fn get_inventory(
        &self,
        ) -> Result<GetInventoryResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().get_inventory(&context).await
    }

    /// Place an order for a pet.
    async fn place_order(
        &self,
        order: Option<models::Order>,
        ) -> Result<PlaceOrderResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().place_order(order, &context).await
    }

    /// Delete purchase order by identifier.
    async fn delete_order(
        &self,
        order_id: i64,
        ) -> Result<DeleteOrderResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().delete_order(order_id, &context).await
    }

    /// Find purchase order by identifier.
    async fn get_order_by_id(
        &self,
        order_id: i64,
        ) -> Result<GetOrderByIdResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().get_order_by_id(order_id, &context).await
    }

    /// Create user.
    async fn create_user(
        &self,
        user: Option<models::User>,
        ) -> Result<CreateUserResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().create_user(user, &context).await
    }

    /// Creates list of users with given input array.
    async fn create_users_with_list_input<'a>(
        &self,
        user: Option<&'a Vec<models::User>>,
        ) -> Result<CreateUsersWithListInputResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().create_users_with_list_input(user, &context).await
    }

    /// Logs user into the system.
    async fn login_user(
        &self,
        username: Option<String>,
        password: Option<String>,
        ) -> Result<LoginUserResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().login_user(username, password, &context).await
    }

    /// Logs out current logged in user session.
    async fn logout_user(
        &self,
        ) -> Result<LogoutUserResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().logout_user(&context).await
    }

    /// Delete user.
    async fn delete_user(
        &self,
        username: String,
        ) -> Result<DeleteUserResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().delete_user(username, &context).await
    }

    /// Get user by user name.
    async fn get_user_by_name(
        &self,
        username: String,
        ) -> Result<GetUserByNameResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().get_user_by_name(username, &context).await
    }

    /// Update user.
    async fn update_user(
        &self,
        username: String,
        user: Option<models::User>,
        ) -> Result<UpdateUserResponse, ApiError>
    {
        let context = self.context().clone();
        self.api().update_user(username, user, &context).await
    }

}


#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

#[cfg(feature = "server")]
pub mod context;

pub mod models;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) mod header;

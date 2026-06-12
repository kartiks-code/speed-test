//! CLI tool driving the API client
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use dialoguer::Confirm;
use log::{debug, info};
// models may be unused if all inputs are primitive types
#[allow(unused_imports)]
use petstore_server::{
    models, ApiNoContext, Client, ContextWrapperExt,
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
    UpdateUserResponse,
};
use simple_logger::SimpleLogger;
use swagger::{AuthData, ContextBuilder, EmptyContext, Push, XSpanIdString};

type ClientContext = swagger::make_context_ty!(
    ContextBuilder,
    EmptyContext,
    Option<AuthData>,
    XSpanIdString
);

#[derive(Parser, Debug)]
#[clap(
    name = "Swagger Petstore - OpenAPI 3.1",
    version = "1.0.12",
    about = "CLI access to Swagger Petstore - OpenAPI 3.1"
)]
struct Cli {
    #[clap(subcommand)]
    operation: Operation,

    /// Address or hostname of the server hosting this API, including optional port
    #[clap(short = 'a', long, default_value = "http://localhost")]
    server_address: String,

    /// Path to the client private key if using client-side TLS authentication
    #[cfg(all(feature = "client-tls", not(any(target_os = "macos", target_os = "windows", target_os = "ios"))))]
    #[clap(long, requires_all(&["client_certificate", "server_certificate"]))]
    client_key: Option<String>,

    /// Path to the client's public certificate associated with the private key
    #[cfg(all(feature = "client-tls", not(any(target_os = "macos", target_os = "windows", target_os = "ios"))))]
    #[clap(long, requires_all(&["client_key", "server_certificate"]))]
    client_certificate: Option<String>,

    /// Path to CA certificate used to authenticate the server
    #[cfg(all(feature = "client-tls", not(any(target_os = "macos", target_os = "windows", target_os = "ios"))))]
    #[clap(long)]
    server_certificate: Option<String>,

    /// If set, write output to file instead of stdout
    #[clap(short, long)]
    output_file: Option<String>,

    #[command(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    /// Don't ask for any confirmation prompts
    #[allow(dead_code)]
    #[clap(short, long)]
    force: bool,

    /// Bearer token if used for authentication
    #[arg(env = "PETSTORE_SERVER_BEARER_TOKEN", hide_env = true)]
    bearer_token: Option<String>,

    /// API key for authentication
    #[arg(long, env = "PETSTORE_SERVER_API_KEY", hide_env = true)]
    api_key: Option<String>,
}

#[derive(Parser, Debug)]
enum Operation {
    /// Add a new pet to the store.
    AddPet {
        /// Create a new pet in the store
        #[clap(value_parser = parse_json::<models::Pet>)]
        pet: models::Pet,
    },
    /// Finds Pets by status.
    FindPetsByStatus {
        /// Status values that need to be considered for filter
        #[clap(value_parser = parse_json::<models::FindPetsByStatusStatusParameter>)]
        status: Option<models::FindPetsByStatusStatusParameter>,
    },
    /// Finds Pets by tags.
    FindPetsByTags {
        /// Tags to filter by
        #[clap(value_parser = parse_json::<Vec<String>>, long)]
        tags: Option<Vec<String>>,
    },
    /// Update an existing pet.
    UpdatePet {
        /// Update an existent pet in the store
        #[clap(value_parser = parse_json::<models::Pet>)]
        pet: models::Pet,
    },
    /// Deletes a pet.
    DeletePet {
        /// Pet id to delete
        pet_id: i64,
        /// 
        api_key: Option<String>,
    },
    /// Find pet by identifier.
    GetPetById {
        /// ID of pet to return
        pet_id: i64,
    },
    /// Updates a pet in the store with form data.
    UpdatePetWithForm {
        /// ID of pet that needs to be updated
        pet_id: i64,
        /// Name of pet that needs to be updated
        name: Option<String>,
        /// Status of pet that needs to be updated
        status: Option<String>,
    },
    /// Uploads an image.
    UploadFile {
        /// ID of pet to update
        pet_id: i64,
        /// Additional Metadata
        additional_metadata: Option<String>,
        #[clap(value_parser = parse_json::<swagger::ByteArray>)]
        body: Option<swagger::ByteArray>,
    },
    /// Returns pet inventories by status.
    GetInventory {
    },
    /// Place an order for a pet.
    PlaceOrder {
        #[clap(value_parser = parse_json::<models::Order>)]
        order: Option<models::Order>,
    },
    /// Delete purchase order by identifier.
    DeleteOrder {
        /// ID of the order that needs to be deleted
        order_id: i64,
    },
    /// Find purchase order by identifier.
    GetOrderById {
        /// ID of order that needs to be fetched
        order_id: i64,
    },
    /// Create user.
    CreateUser {
        /// Created user object
        #[clap(value_parser = parse_json::<models::User>)]
        user: Option<models::User>,
    },
    /// Creates list of users with given input array.
    CreateUsersWithListInput {
        #[clap(value_parser = parse_json::<Vec<models::User>>, long)]
        user: Option<Vec<models::User>>,
    },
    /// Logs user into the system.
    LoginUser {
        /// The user name for login
        username: Option<String>,
        /// The password for login in clear text
        password: Option<String>,
    },
    /// Logs out current logged in user session.
    LogoutUser {
    },
    /// Delete user.
    DeleteUser {
        /// The name that needs to be deleted
        username: String,
    },
    /// Get user by user name.
    GetUserByName {
        /// The name that needs to be fetched. Use user1 for testing
        username: String,
    },
    /// Update user.
    UpdateUser {
        /// name that need to be deleted
        username: String,
        /// Update an existent user in the store
        #[clap(value_parser = parse_json::<models::User>)]
        user: Option<models::User>,
    },
}

// On Linux/Unix with OpenSSL (client-tls feature), support certificate pinning and mutual TLS
#[cfg(all(feature = "client-tls", not(any(target_os = "macos", target_os = "windows", target_os = "ios"))))]
fn create_client(args: &Cli, context: ClientContext) -> Result<Box<dyn ApiNoContext<ClientContext>>> {
    if args.client_certificate.is_some() {
        debug!("Using mutual TLS");
        let client = Client::try_new_https_mutual(
            &args.server_address,
            args.server_certificate.clone().unwrap(),
            args.client_key.clone().unwrap(),
            args.client_certificate.clone().unwrap(),
        )
        .context("Failed to create HTTPS client")?;
        Ok(Box::new(client.with_context(context)))
    } else if args.server_certificate.is_some() {
        debug!("Using TLS with pinned server certificate");
        let client =
            Client::try_new_https_pinned(&args.server_address, args.server_certificate.clone().unwrap())
                .context("Failed to create HTTPS client")?;
        Ok(Box::new(client.with_context(context)))
    } else {
        debug!("Using client without certificates");
        let client =
            Client::try_new(&args.server_address).context("Failed to create HTTP(S) client")?;
        Ok(Box::new(client.with_context(context)))
    }
}

// On macOS/Windows/iOS or without client-tls feature, use simple client (no cert pinning/mutual TLS)
#[cfg(any(
    not(feature = "client-tls"),
    all(feature = "client-tls", any(target_os = "macos", target_os = "windows", target_os = "ios"))
))]
fn create_client(args: &Cli, context: ClientContext) -> Result<Box<dyn ApiNoContext<ClientContext>>> {
    // Client::try_new() automatically detects the URL scheme (http:// or https://)
    // and creates the appropriate client.
    // Note: Certificate pinning and mutual TLS are only available on Linux/Unix with OpenSSL
    let client =
        Client::try_new(&args.server_address).context("Failed to create HTTP(S) client")?;
    Ok(Box::new(client.with_context(context)))
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    if let Some(log_level) = args.verbosity.log_level() {
        SimpleLogger::new().with_level(log_level.to_level_filter()).init()?;
    }

    debug!("Arguments: {:?}", &args);

    let mut auth_data: Option<AuthData> = None;

    if let Some(ref bearer_token) = args.bearer_token {
        debug!("Using bearer token");
        auth_data = AuthData::bearer(bearer_token);
    }
    if let Some(ref api_key) = args.api_key {
        debug!("Using API key");
        auth_data = Some(AuthData::apikey(api_key));
    }

    #[allow(trivial_casts)]
    let context = swagger::make_context!(
        ContextBuilder,
        EmptyContext,
        auth_data,
        XSpanIdString::default()
    );

    let client = create_client(&args, context)?;

    let result = match args.operation {
        Operation::AddPet {
            pet,
        } => {
            info!("Performing a AddPet request");

            let result = client.add_pet(
                pet,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                AddPetResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                AddPetResponse::InvalidInput
                => "InvalidInput\n".to_string()
                    ,
                AddPetResponse::ValidationException
                => "ValidationException\n".to_string()
                    ,
                AddPetResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::FindPetsByStatus {
            status,
        } => {
            info!("Performing a FindPetsByStatus request");

            let result = client.find_pets_by_status(
                status,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                FindPetsByStatusResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                FindPetsByStatusResponse::InvalidStatusValue
                => "InvalidStatusValue\n".to_string()
                    ,
                FindPetsByStatusResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::FindPetsByTags {
            tags,
        } => {
            info!("Performing a FindPetsByTags request");

            let result = client.find_pets_by_tags(
                tags.as_ref(),
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                FindPetsByTagsResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                FindPetsByTagsResponse::InvalidTagValue
                => "InvalidTagValue\n".to_string()
                    ,
                FindPetsByTagsResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::UpdatePet {
            pet,
        } => {
            info!("Performing a UpdatePet request");

            let result = client.update_pet(
                pet,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                UpdatePetResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                UpdatePetResponse::InvalidIDSupplied
                => "InvalidIDSupplied\n".to_string()
                    ,
                UpdatePetResponse::PetNotFound
                => "PetNotFound\n".to_string()
                    ,
                UpdatePetResponse::ValidationException
                => "ValidationException\n".to_string()
                    ,
                UpdatePetResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::DeletePet {
            pet_id,
            api_key,
        } => {
            prompt(args.force, "This will delete the given entry, are you sure?")?;
            info!("Performing a DeletePet request on {:?}", (
                &pet_id
            ));

            let result = client.delete_pet(
                pet_id,
                api_key,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                DeletePetResponse::SuccessfulOperation
                => "SuccessfulOperation\n".to_string()
                    ,
                DeletePetResponse::InvalidPetValue
                => "InvalidPetValue\n".to_string()
                    ,
                DeletePetResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::GetPetById {
            pet_id,
        } => {
            info!("Performing a GetPetById request on {:?}", (
                &pet_id
            ));

            let result = client.get_pet_by_id(
                pet_id,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                GetPetByIdResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                GetPetByIdResponse::InvalidIDSupplied
                => "InvalidIDSupplied\n".to_string()
                    ,
                GetPetByIdResponse::PetNotFound
                => "PetNotFound\n".to_string()
                    ,
                GetPetByIdResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::UpdatePetWithForm {
            pet_id,
            name,
            status,
        } => {
            info!("Performing a UpdatePetWithForm request on {:?}", (
                &pet_id
            ));

            let result = client.update_pet_with_form(
                pet_id,
                name,
                status,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                UpdatePetWithFormResponse::SuccessfullyUpdated
                => "SuccessfullyUpdated\n".to_string()
                    ,
                UpdatePetWithFormResponse::InvalidInput
                => "InvalidInput\n".to_string()
                    ,
                UpdatePetWithFormResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::UploadFile {
            pet_id,
            additional_metadata,
            body,
        } => {
            info!("Performing a UploadFile request on {:?}", (
                &pet_id
            ));

            let result = client.upload_file(
                pet_id,
                additional_metadata,
                body,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                UploadFileResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                UploadFileResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::GetInventory {
        } => {
            info!("Performing a GetInventory request");

            let result = client.get_inventory(
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                GetInventoryResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                GetInventoryResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::PlaceOrder {
            order,
        } => {
            info!("Performing a PlaceOrder request");

            let result = client.place_order(
                order,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                PlaceOrderResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                PlaceOrderResponse::InvalidInput
                => "InvalidInput\n".to_string()
                    ,
                PlaceOrderResponse::ValidationException
                => "ValidationException\n".to_string()
                    ,
                PlaceOrderResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::DeleteOrder {
            order_id,
        } => {
            prompt(args.force, "This will delete the given entry, are you sure?")?;
            info!("Performing a DeleteOrder request on {:?}", (
                &order_id
            ));

            let result = client.delete_order(
                order_id,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                DeleteOrderResponse::SuccessfulOperation
                => "SuccessfulOperation\n".to_string()
                    ,
                DeleteOrderResponse::InvalidIDSupplied
                => "InvalidIDSupplied\n".to_string()
                    ,
                DeleteOrderResponse::OrderNotFound
                => "OrderNotFound\n".to_string()
                    ,
                DeleteOrderResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::GetOrderById {
            order_id,
        } => {
            info!("Performing a GetOrderById request on {:?}", (
                &order_id
            ));

            let result = client.get_order_by_id(
                order_id,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                GetOrderByIdResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                GetOrderByIdResponse::InvalidIDSupplied
                => "InvalidIDSupplied\n".to_string()
                    ,
                GetOrderByIdResponse::OrderNotFound
                => "OrderNotFound\n".to_string()
                    ,
                GetOrderByIdResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::CreateUser {
            user,
        } => {
            info!("Performing a CreateUser request");

            let result = client.create_user(
                user,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                CreateUserResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                CreateUserResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::CreateUsersWithListInput {
            user,
        } => {
            info!("Performing a CreateUsersWithListInput request");

            let result = client.create_users_with_list_input(
                user.as_ref(),
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                CreateUsersWithListInputResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                CreateUsersWithListInputResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::LoginUser {
            username,
            password,
        } => {
            info!("Performing a LoginUser request");

            let result = client.login_user(
                username,
                password,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                LoginUserResponse::SuccessfulOperation
                {
                    body,
                    x_rate_limit,
                    x_expires_after,
                }
                => "SuccessfulOperation\n".to_string()
                   +
                    &format!("body: {}\n", serde_json::to_string_pretty(&body)?) +
                    &format!(
                        "x_rate_limit: {}\n",
                        serde_json::to_string_pretty(&x_rate_limit)?
                    ) +
                    &format!(
                        "x_expires_after: {}\n",
                        serde_json::to_string_pretty(&x_expires_after)?
                    ),
                LoginUserResponse::InvalidUsername
                => "InvalidUsername\n".to_string()
                    ,
                LoginUserResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::LogoutUser {
        } => {
            info!("Performing a LogoutUser request");

            let result = client.logout_user(
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                LogoutUserResponse::SuccessfulOperation
                => "SuccessfulOperation\n".to_string()
                    ,
                LogoutUserResponse::SuccessfulOperation_2
                => "SuccessfulOperation_2\n".to_string()
                    ,
            }
        }
        Operation::DeleteUser {
            username,
        } => {
            prompt(args.force, "This will delete the given entry, are you sure?")?;
            info!("Performing a DeleteUser request on {:?}", (
                &username
            ));

            let result = client.delete_user(
                username,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                DeleteUserResponse::SuccessfulOperation
                => "SuccessfulOperation\n".to_string()
                    ,
                DeleteUserResponse::InvalidUsernameSupplied
                => "InvalidUsernameSupplied\n".to_string()
                    ,
                DeleteUserResponse::UserNotFound
                => "UserNotFound\n".to_string()
                    ,
                DeleteUserResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::GetUserByName {
            username,
        } => {
            info!("Performing a GetUserByName request on {:?}", (
                &username
            ));

            let result = client.get_user_by_name(
                username,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                GetUserByNameResponse::SuccessfulOperation
                (body)
                => "SuccessfulOperation\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
                GetUserByNameResponse::InvalidUsernameSupplied
                => "InvalidUsernameSupplied\n".to_string()
                    ,
                GetUserByNameResponse::UserNotFound
                => "UserNotFound\n".to_string()
                    ,
                GetUserByNameResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
        Operation::UpdateUser {
            username,
            user,
        } => {
            info!("Performing a UpdateUser request on {:?}", (
                &username
            ));

            let result = client.update_user(
                username,
                user,
            ).await?;
            debug!("Result: {:?}", result);

            match result {
                UpdateUserResponse::SuccessfulOperation
                => "SuccessfulOperation\n".to_string()
                    ,
                UpdateUserResponse::UnexpectedError
                (body)
                => "UnexpectedError\n".to_string()
                   +
                    &serde_json::to_string_pretty(&body)?,
            }
        }
    };

    if let Some(output_file) = args.output_file {
        std::fs::write(output_file, result)?
    } else {
        println!("{}", result);
    }
    Ok(())
}

fn prompt(force: bool, text: &str) -> Result<()> {
    if force || Confirm::new().with_prompt(text).interact()? {
        Ok(())
    } else {
        Err(anyhow!("Aborting"))
    }
}

// May be unused if all inputs are primitive types
#[allow(dead_code)]
fn parse_json<T: serde::de::DeserializeOwned>(json_string: &str) -> Result<T> {
    serde_json::from_str(json_string).map_err(|err| anyhow!("Error parsing input: {}", err))
}

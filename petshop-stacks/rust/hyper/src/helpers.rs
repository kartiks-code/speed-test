//! Pure row-to-model conversion helpers.
//!
//! These functions translate raw column values (as returned from the database
//! into Rust types by sqlx) into the generated model structs.  They are
//! intentionally database-free so they can be unit-tested without a live
//! PostgreSQL connection.

use crate::models;

/// Convert database column values into a [`models::Pet`].
///
/// - `category_text`: TEXT column containing a JSON-encoded [`models::Category`] (may be `NULL`).
/// - `photo_urls_text`: JSON array column read as TEXT; malformed JSON falls back to `[]`.
/// - `tags_text`: JSON array column read as TEXT (may be `NULL`).
/// - `status_text`: enum column cast to TEXT (may be `NULL`); unknown values become `None`.
pub fn row_to_pet(
    id: Option<i64>,
    name: String,
    category_text: Option<String>,
    photo_urls_text: String,
    tags_text: Option<String>,
    status_text: Option<String>,
) -> models::Pet {
    let category = category_text
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());
    let photo_urls: Vec<String> =
        serde_json::from_str(&photo_urls_text).unwrap_or_default();
    let tags: Option<Vec<models::Tag>> = tags_text
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok());
    let status = status_text
        .as_deref()
        .and_then(|s| s.parse::<models::PetStatus>().ok());

    models::Pet {
        id,
        name,
        category,
        photo_urls,
        tags,
        status,
    }
}

/// Convert database column values into a [`models::Order`].
///
/// `ship_date_naive` is a `TIMESTAMP WITHOUT TIME ZONE` column; it is assumed
/// to be UTC and converted to a `DateTime<Utc>`.
pub fn row_to_order(
    id: Option<i64>,
    pet_id: Option<i64>,
    quantity: Option<i32>,
    ship_date_naive: Option<chrono::NaiveDateTime>,
    status_text: Option<String>,
    complete: Option<bool>,
) -> models::Order {
    let ship_date = ship_date_naive.map(|nd| {
        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(nd, chrono::Utc)
    });
    let status = status_text
        .as_deref()
        .and_then(|s| s.parse::<models::OrderStatus>().ok());
    models::Order {
        id,
        pet_id,
        quantity,
        ship_date,
        status,
        complete,
    }
}

/// Convert database column values into a [`models::User`].
///
/// All columns are mapped 1-to-1; no conversion is required.
pub fn row_to_user(
    id: Option<i64>,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    phone: Option<String>,
    user_status: Option<i32>,
) -> models::User {
    models::User {
        id,
        username,
        first_name,
        last_name,
        email,
        password,
        phone,
        user_status,
    }
}

// ---------------------------------------------------------------------------
// Unit tests — database-free
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{row_to_order, row_to_pet, row_to_user};
    use crate::models;

    // --- Pet row -> model -----------------------------------------------

    #[test]
    fn row_to_pet_decodes_all_columns() {
        let pet = row_to_pet(
            Some(7),
            "Fido".to_string(),
            Some(r#"{"id":1,"name":"dogs"}"#.to_string()),
            r#"["http://example.com/a.jpg","http://example.com/b.jpg"]"#.to_string(),
            Some(r#"[{"id":3,"name":"cute"}]"#.to_string()),
            Some("available".to_string()),
        );

        assert_eq!(pet.id, Some(7));
        assert_eq!(pet.name, "Fido");
        assert_eq!(
            pet.category,
            Some(models::Category {
                id: Some(1),
                name: Some("dogs".to_string()),
            })
        );
        assert_eq!(
            pet.photo_urls,
            vec![
                "http://example.com/a.jpg".to_string(),
                "http://example.com/b.jpg".to_string(),
            ]
        );
        assert_eq!(
            pet.tags,
            Some(vec![models::Tag {
                id: Some(3),
                name: Some("cute".to_string()),
            }])
        );
        assert_eq!(pet.status, Some(models::PetStatus::Available));
    }

    #[test]
    fn row_to_pet_status_pending() {
        let pet = row_to_pet(
            Some(1),
            "Rex".to_string(),
            None,
            "[]".to_string(),
            None,
            Some("pending".to_string()),
        );
        assert_eq!(pet.status, Some(models::PetStatus::Pending));
    }

    #[test]
    fn row_to_pet_status_sold() {
        let pet = row_to_pet(
            Some(2),
            "Buddy".to_string(),
            None,
            "[]".to_string(),
            None,
            Some("sold".to_string()),
        );
        assert_eq!(pet.status, Some(models::PetStatus::Sold));
    }

    #[test]
    fn row_to_pet_handles_null_optionals() {
        let pet = row_to_pet(None, "Rex".to_string(), None, "[]".to_string(), None, None);

        assert_eq!(pet.id, None);
        assert_eq!(pet.category, None);
        assert!(pet.photo_urls.is_empty());
        assert_eq!(pet.tags, None);
        assert_eq!(pet.status, None);
        assert_eq!(pet.name, "Rex");
    }

    #[test]
    fn row_to_pet_empty_tags_list() {
        let pet = row_to_pet(
            Some(5),
            "Kitty".to_string(),
            None,
            "[]".to_string(),
            Some("[]".to_string()),
            None,
        );
        assert_eq!(pet.tags, Some(vec![]));
    }

    #[test]
    fn row_to_pet_multiple_tags() {
        let pet = row_to_pet(
            Some(3),
            "Max".to_string(),
            None,
            r#"["http://x.com/1.jpg"]"#.to_string(),
            Some(r#"[{"id":1,"name":"alpha"},{"id":2,"name":"beta"}]"#.to_string()),
            Some("available".to_string()),
        );
        let tags = pet.tags.unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].id, Some(1));
        assert_eq!(tags[0].name, Some("alpha".to_string()));
        assert_eq!(tags[1].id, Some(2));
        assert_eq!(tags[1].name, Some("beta".to_string()));
    }

    #[test]
    fn row_to_pet_preserves_id() {
        let pet = row_to_pet(
            Some(42),
            "Spot".to_string(),
            None,
            "[]".to_string(),
            None,
            None,
        );
        assert_eq!(pet.id, Some(42));
    }

    #[test]
    fn row_to_pet_none_id() {
        let pet = row_to_pet(None, "Ghost".to_string(), None, "[]".to_string(), None, None);
        assert_eq!(pet.id, None);
    }

    #[test]
    fn row_to_pet_tolerates_malformed_json_columns() {
        let pet = row_to_pet(
            Some(1),
            "Spot".to_string(),
            Some("not json".to_string()),
            "also not json".to_string(),
            Some("{".to_string()),
            Some("teleporting".to_string()),
        );

        assert!(pet.photo_urls.is_empty());
        assert_eq!(pet.category, None);
        assert_eq!(pet.tags, None);
        assert_eq!(pet.status, None);
    }

    #[test]
    fn row_to_pet_category_decoded_correctly() {
        let cat_json = r#"{"id":99,"name":"reptiles"}"#.to_string();
        let pet = row_to_pet(
            Some(10),
            "Iggy".to_string(),
            Some(cat_json),
            "[]".to_string(),
            None,
            None,
        );
        assert_eq!(
            pet.category,
            Some(models::Category {
                id: Some(99),
                name: Some("reptiles".to_string()),
            })
        );
    }

    #[test]
    fn row_to_pet_photo_urls_decoded() {
        let pet = row_to_pet(
            Some(1),
            "Pic".to_string(),
            None,
            r#"["http://a.com/1.jpg","http://b.com/2.jpg","http://c.com/3.jpg"]"#.to_string(),
            None,
            None,
        );
        assert_eq!(pet.photo_urls.len(), 3);
        assert_eq!(pet.photo_urls[0], "http://a.com/1.jpg");
        assert_eq!(pet.photo_urls[2], "http://c.com/3.jpg");
    }

    // --- Order row -> model ---------------------------------------------

    #[test]
    fn row_to_order_decodes_and_assumes_utc_for_ship_date() {
        let naive = chrono::NaiveDate::from_ymd_opt(2024, 1, 2)
            .unwrap()
            .and_hms_opt(3, 4, 5)
            .unwrap();

        let order = row_to_order(
            Some(10),
            Some(20),
            Some(2),
            Some(naive),
            Some("placed".to_string()),
            Some(true),
        );

        assert_eq!(order.id, Some(10));
        assert_eq!(order.pet_id, Some(20));
        assert_eq!(order.quantity, Some(2));
        assert_eq!(order.status, Some(models::OrderStatus::Placed));
        assert_eq!(order.complete, Some(true));

        let ship_date = order.ship_date.expect("ship_date should be set");
        assert_eq!(ship_date.naive_utc(), naive);
    }

    #[test]
    fn row_to_order_status_approved() {
        let order = row_to_order(Some(1), None, None, None, Some("approved".to_string()), None);
        assert_eq!(order.status, Some(models::OrderStatus::Approved));
    }

    #[test]
    fn row_to_order_status_delivered() {
        let order = row_to_order(
            Some(2),
            None,
            None,
            None,
            Some("delivered".to_string()),
            None,
        );
        assert_eq!(order.status, Some(models::OrderStatus::Delivered));
    }

    #[test]
    fn row_to_order_handles_null_ship_date_and_status() {
        let order = row_to_order(Some(1), None, None, None, None, None);
        assert_eq!(order.ship_date, None);
        assert_eq!(order.status, None);
    }

    #[test]
    fn row_to_order_complete_false() {
        let order = row_to_order(Some(1), Some(5), Some(3), None, None, Some(false));
        assert_eq!(order.complete, Some(false));
        assert_eq!(order.pet_id, Some(5));
        assert_eq!(order.quantity, Some(3));
    }

    #[test]
    fn row_to_order_complete_none() {
        let order = row_to_order(Some(1), None, None, None, None, None);
        assert_eq!(order.complete, None);
    }

    #[test]
    fn row_to_order_invalid_status_becomes_none() {
        let order = row_to_order(
            Some(1),
            None,
            None,
            None,
            Some("nonsense".to_string()),
            None,
        );
        assert_eq!(order.status, None);
    }

    #[test]
    fn row_to_order_ship_date_preserved_to_seconds() {
        let naive = chrono::NaiveDate::from_ymd_opt(2030, 12, 31)
            .unwrap()
            .and_hms_opt(23, 59, 58)
            .unwrap();
        let order = row_to_order(Some(1), None, None, Some(naive), None, None);
        let ship = order.ship_date.unwrap();
        assert_eq!(ship.naive_utc(), naive);
    }

    // --- User row -> model ----------------------------------------------

    #[test]
    fn row_to_user_maps_fields_directly() {
        let user = row_to_user(
            Some(5),
            Some("alice".to_string()),
            Some("Alice".to_string()),
            Some("Smith".to_string()),
            Some("alice@example.com".to_string()),
            Some("secret".to_string()),
            Some("555-1234".to_string()),
            Some(1),
        );

        assert_eq!(user.id, Some(5));
        assert_eq!(user.username, Some("alice".to_string()));
        assert_eq!(user.first_name, Some("Alice".to_string()));
        assert_eq!(user.last_name, Some("Smith".to_string()));
        assert_eq!(user.email, Some("alice@example.com".to_string()));
        assert_eq!(user.password, Some("secret".to_string()));
        assert_eq!(user.phone, Some("555-1234".to_string()));
        assert_eq!(user.user_status, Some(1));
    }

    #[test]
    fn row_to_user_all_none() {
        let user = row_to_user(None, None, None, None, None, None, None, None);
        assert_eq!(user.id, None);
        assert_eq!(user.username, None);
        assert_eq!(user.first_name, None);
        assert_eq!(user.last_name, None);
        assert_eq!(user.email, None);
        assert_eq!(user.password, None);
        assert_eq!(user.phone, None);
        assert_eq!(user.user_status, None);
    }

    #[test]
    fn row_to_user_id_only() {
        let user = row_to_user(Some(99), None, None, None, None, None, None, None);
        assert_eq!(user.id, Some(99));
        assert_eq!(user.username, None);
    }

    #[test]
    fn row_to_user_status_zero() {
        let user = row_to_user(None, None, None, None, None, None, None, Some(0));
        assert_eq!(user.user_status, Some(0));
    }

    #[test]
    fn row_to_user_negative_status() {
        let user = row_to_user(None, None, None, None, None, None, None, Some(-1));
        assert_eq!(user.user_status, Some(-1));
    }

    // --- Enum <-> string conversions ------------------------------------

    #[test]
    fn pet_status_display_and_fromstr_round_trip() {
        for (variant, text) in [
            (models::PetStatus::Available, "available"),
            (models::PetStatus::Pending, "pending"),
            (models::PetStatus::Sold, "sold"),
        ] {
            assert_eq!(variant.to_string(), text);
            assert_eq!(text.parse::<models::PetStatus>().unwrap(), variant);
        }
    }

    #[test]
    fn order_status_display_and_fromstr_round_trip() {
        for (variant, text) in [
            (models::OrderStatus::Placed, "placed"),
            (models::OrderStatus::Approved, "approved"),
            (models::OrderStatus::Delivered, "delivered"),
        ] {
            assert_eq!(variant.to_string(), text);
            assert_eq!(text.parse::<models::OrderStatus>().unwrap(), variant);
        }
    }

    #[test]
    fn invalid_enum_strings_are_rejected() {
        assert!("nonsense".parse::<models::PetStatus>().is_err());
        assert!("nonsense".parse::<models::OrderStatus>().is_err());
        assert!("Available".parse::<models::PetStatus>().is_err());
        assert!("Placed".parse::<models::OrderStatus>().is_err());
        assert!("AVAILABLE".parse::<models::PetStatus>().is_err());
    }

    // --- JSON encoding conventions --------------------------------------

    #[test]
    fn photo_urls_json_round_trip() {
        let urls = vec!["http://a/1.jpg".to_string(), "http://a/2.jpg".to_string()];
        let json = serde_json::to_string(&urls).unwrap();
        assert_eq!(json, r#"["http://a/1.jpg","http://a/2.jpg"]"#);

        let decoded: Vec<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, urls);
    }

    #[test]
    fn tags_json_round_trip() {
        let tags = vec![
            models::Tag {
                id: Some(1),
                name: Some("a".to_string()),
            },
            models::Tag {
                id: Some(2),
                name: Some("b".to_string()),
            },
        ];
        let json = serde_json::to_string(&tags).unwrap();
        let decoded: Vec<models::Tag> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, tags);
    }

    #[test]
    fn category_serializes_as_json_string_and_round_trips_through_row_to_pet() {
        let category = models::Category {
            id: Some(42),
            name: Some("reptiles".to_string()),
        };
        let stored = serde_json::to_string(&category).unwrap();
        assert_eq!(stored, r#"{"id":42,"name":"reptiles"}"#);

        let pet = row_to_pet(
            Some(1),
            "Iggy".to_string(),
            Some(stored),
            "[]".to_string(),
            None,
            None,
        );
        assert_eq!(pet.category, Some(category));
    }

    // --- chrono NaiveDateTime conversion (ship_date) --------------------

    #[test]
    fn ship_date_naive_utc_conversion_round_trips() {
        let original = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            chrono::NaiveDate::from_ymd_opt(2030, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 58)
                .unwrap(),
            chrono::Utc,
        );

        let naive = original.naive_utc();
        let restored =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive, chrono::Utc);

        assert_eq!(restored, original);
    }
}

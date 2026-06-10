--
-- "Swagger Petstore - OpenAPI 3.1"
-- Prepared SQL queries for 'User' definition.
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--


--
-- SELECT template for table 'user'
--
SELECT "id", username, first_name, last_name, email, "password", phone, user_status FROM "user" WHERE 1=1;

--
-- INSERT template for table 'user'
--
INSERT INTO "user" ("id", username, first_name, last_name, email, "password", phone, user_status) VALUES (:id, :username, :first_name, :last_name, :email, :password, :phone, :user_status);

--
-- UPDATE template for table 'user'
--
UPDATE "user" SET "id" = :id, first_name = :first_name, last_name = :last_name, email = :email, "password" = :password, phone = :phone, user_status = :user_status WHERE username = :username;

--
-- DELETE template for table 'user'
--
DELETE FROM "user" WHERE username = :username;


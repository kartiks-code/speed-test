--
-- "Swagger Petstore - OpenAPI 3.1"
-- Prepared SQL queries for 'Customer' definition.
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--


--
-- SELECT template for table 'customer'
--
SELECT "id", username, address FROM customer WHERE 1=1;

--
-- INSERT template for table 'customer'
--
INSERT INTO customer ("id", username, address) VALUES (:id, :username, :address);

--
-- UPDATE template for table 'customer'
--
UPDATE customer SET username = :username, address = :address WHERE "id" = :id;

--
-- DELETE template for table 'customer'
--
DELETE FROM customer WHERE "id" = :id;


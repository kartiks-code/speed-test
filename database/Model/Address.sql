--
-- "Swagger Petstore - OpenAPI 3.1"
-- Prepared SQL queries for 'Address' definition.
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--


--
-- SELECT template for table 'address'
--
SELECT street, city, "state", zip FROM address WHERE 1=1;

--
-- INSERT template for table 'address'
--
INSERT INTO address (street, city, "state", zip) VALUES (:street, :city, :state, :zip);

--
-- UPDATE template for table 'address'
--
UPDATE address SET street = :street, city = :city, "state" = :state, zip = :zip WHERE 1=2;

--
-- DELETE template for table 'address'
--
DELETE FROM address WHERE 1=2;


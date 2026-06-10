--
-- "Swagger Petstore - OpenAPI 3.1"
-- Prepared SQL queries for 'Category' definition.
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--


--
-- SELECT template for table 'category'
--
SELECT "id", "name" FROM category WHERE 1=1;

--
-- INSERT template for table 'category'
--
INSERT INTO category ("id", "name") VALUES (:id, :name);

--
-- UPDATE template for table 'category'
--
UPDATE category SET "name" = :name WHERE "id" = :id;

--
-- DELETE template for table 'category'
--
DELETE FROM category WHERE "id" = :id;


--
-- "Swagger Petstore - OpenAPI 3.1"
-- Prepared SQL queries for 'Tag' definition.
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--


--
-- SELECT template for table 'tag'
--
SELECT "id", "name" FROM tag WHERE 1=1;

--
-- INSERT template for table 'tag'
--
INSERT INTO tag ("id", "name") VALUES (:id, :name);

--
-- UPDATE template for table 'tag'
--
UPDATE tag SET "name" = :name WHERE "id" = :id;

--
-- DELETE template for table 'tag'
--
DELETE FROM tag WHERE "id" = :id;


--
-- "Swagger Petstore - OpenAPI 3.1"
-- Prepared SQL queries for 'Error' definition.
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--


--
-- SELECT template for table 'error'
--
SELECT code, message FROM "error" WHERE 1=1;

--
-- INSERT template for table 'error'
--
INSERT INTO "error" (code, message) VALUES (:code, :message);

--
-- UPDATE template for table 'error'
--
UPDATE "error" SET code = :code, message = :message WHERE 1=2;

--
-- DELETE template for table 'error'
--
DELETE FROM "error" WHERE 1=2;


--
-- "Swagger Petstore - OpenAPI 3.1"
-- Prepared SQL queries for 'Pet' definition.
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--


--
-- SELECT template for table 'pet'
--
SELECT "id", "name", category, photo_urls, tags, status FROM pet WHERE 1=1;

--
-- INSERT template for table 'pet'
--
INSERT INTO pet ("id", "name", category, photo_urls, tags, status) VALUES (:id, :name, :category, :photo_urls, :tags, :status);

--
-- UPDATE template for table 'pet'
--
UPDATE pet SET "name" = :name, category = :category, photo_urls = :photo_urls, tags = :tags, status = :status WHERE "id" = :id;

--
-- DELETE template for table 'pet'
--
DELETE FROM pet WHERE "id" = :id;


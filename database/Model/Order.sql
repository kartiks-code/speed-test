--
-- "Swagger Petstore - OpenAPI 3.1"
-- Prepared SQL queries for 'Order' definition.
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--


--
-- SELECT template for table 'order'
--
SELECT "id", pet_id, quantity, ship_date, status, complete FROM "order" WHERE 1=1;

--
-- INSERT template for table 'order'
--
INSERT INTO "order" ("id", pet_id, quantity, ship_date, status, complete) VALUES (:id, :pet_id, :quantity, :ship_date, :status, :complete);

--
-- UPDATE template for table 'order'
--
UPDATE "order" SET pet_id = :pet_id, quantity = :quantity, ship_date = :ship_date, status = :status, complete = :complete WHERE "id" = :id;

--
-- DELETE template for table 'order'
--
DELETE FROM "order" WHERE "id" = :id;


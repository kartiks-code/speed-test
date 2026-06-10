--
-- Schema objects for PostgreSQL
-- "Swagger Petstore - OpenAPI 3.1"
-- Created using 'openapi-generator' ('postgresql-schema' generator)
-- (https://openapi-generator.tech/docs/generators/postgresql-schema)
--

--
-- DROP OBJECTS
-- (remove comment prefix to start using DROP commands)
--
-- TABLES
--
-- DROP TABLE IF EXISTS address;
-- DROP TABLE IF EXISTS api_response;
-- DROP TABLE IF EXISTS category;
-- DROP TABLE IF EXISTS customer;
-- DROP TABLE IF EXISTS "error";
-- DROP TABLE IF EXISTS "order";
-- DROP TABLE IF EXISTS pet;
-- DROP TABLE IF EXISTS tag;
-- DROP TABLE IF EXISTS "user";

--
-- TYPES
--
-- DROP TYPE IF EXISTS order_status;
-- DROP TYPE IF EXISTS pet_status;


--
-- CREATE OBJECTS
--
-- TYPES
--
CREATE TYPE order_status AS ENUM('placed', 'approved', 'delivered');
CREATE TYPE pet_status AS ENUM('available', 'pending', 'sold');

--
-- TABLES
--
--
-- Table 'address' generated from model 'Address'
--
CREATE TABLE IF NOT EXISTS address (
    street TEXT DEFAULT NULL,
    city TEXT DEFAULT NULL,
    "state" TEXT DEFAULT NULL,
    zip TEXT DEFAULT NULL
);
COMMENT ON TABLE address IS 'Original model name - Address.';

--
-- Table 'api_response' generated from model 'ApiResponse'
--
CREATE TABLE IF NOT EXISTS api_response (
    code INTEGER DEFAULT NULL,
    "type" TEXT DEFAULT NULL,
    message TEXT DEFAULT NULL
);
COMMENT ON TABLE api_response IS 'Original model name - ApiResponse.';

--
-- Table 'category' generated from model 'Category'
--
CREATE TABLE IF NOT EXISTS category (
    "id" BIGINT PRIMARY KEY,
    "name" TEXT DEFAULT NULL
);
COMMENT ON TABLE category IS 'Original model name - Category.';

--
-- Table 'customer' generated from model 'Customer'
--
CREATE TABLE IF NOT EXISTS customer (
    "id" BIGINT PRIMARY KEY,
    username TEXT DEFAULT NULL,
    address JSON DEFAULT NULL
);
COMMENT ON TABLE customer IS 'Original model name - Customer.';

--
-- Table 'error' generated from model 'Error'
--
CREATE TABLE IF NOT EXISTS "error" (
    code TEXT NOT NULL,
    message TEXT NOT NULL
);
COMMENT ON TABLE "error" IS 'Original model name - Error.';

--
-- Table 'order' generated from model 'Order'
--
CREATE TABLE IF NOT EXISTS "order" (
    "id" BIGINT PRIMARY KEY,
    pet_id BIGINT DEFAULT NULL,
    quantity INTEGER DEFAULT NULL,
    ship_date TIMESTAMP DEFAULT NULL,
    status order_status DEFAULT NULL,
    complete BOOLEAN DEFAULT NULL
);
COMMENT ON TABLE "order" IS 'Original model name - Order.';
COMMENT ON COLUMN "order".pet_id IS 'Original param name - petId.';
COMMENT ON COLUMN "order".ship_date IS 'Original param name - shipDate.';
COMMENT ON COLUMN "order".status IS 'Order Status';

--
-- Table 'pet' generated from model 'Pet'
--
CREATE TABLE IF NOT EXISTS pet (
    "id" BIGINT PRIMARY KEY,
    "name" TEXT NOT NULL,
    category TEXT DEFAULT NULL,
    photo_urls JSON NOT NULL,
    tags JSON DEFAULT NULL,
    status pet_status DEFAULT NULL
);
COMMENT ON TABLE pet IS 'Original model name - Pet.';
COMMENT ON COLUMN pet.photo_urls IS 'Original param name - photoUrls.';
COMMENT ON COLUMN pet.status IS 'pet status in the store';

--
-- Table 'tag' generated from model 'Tag'
--
CREATE TABLE IF NOT EXISTS tag (
    "id" BIGINT PRIMARY KEY,
    "name" TEXT DEFAULT NULL
);
COMMENT ON TABLE tag IS 'Original model name - Tag.';

--
-- Table 'user' generated from model 'User'
--
CREATE TABLE IF NOT EXISTS "user" (
    "id" BIGINT DEFAULT NULL,
    username TEXT PRIMARY KEY,
    first_name TEXT DEFAULT NULL,
    last_name TEXT DEFAULT NULL,
    email TEXT DEFAULT NULL,
    "password" TEXT DEFAULT NULL,
    phone TEXT DEFAULT NULL,
    user_status INTEGER DEFAULT NULL
);
COMMENT ON TABLE "user" IS 'Original model name - User.';
COMMENT ON COLUMN "user".first_name IS 'Original param name - firstName.';
COMMENT ON COLUMN "user".last_name IS 'Original param name - lastName.';
COMMENT ON COLUMN "user".user_status IS 'User Status. Original param name - userStatus.';

--
-- CRUD identity constraints
--
CREATE UNIQUE INDEX IF NOT EXISTS pet_id_unique_idx ON pet ("id");
CREATE UNIQUE INDEX IF NOT EXISTS order_id_unique_idx ON "order" ("id");
CREATE UNIQUE INDEX IF NOT EXISTS user_username_unique_idx ON "user" (username);

--
-- PRIMARY KEY CONSTRAINTS
-- These blocks add keys for databases created before the primary keys were
-- added to the generated CREATE TABLE statements above.
--
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'category_pkey' AND conrelid = 'category'::regclass
    ) THEN
        ALTER TABLE category ALTER COLUMN "id" SET NOT NULL;
        ALTER TABLE category ADD CONSTRAINT category_pkey PRIMARY KEY ("id");
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'customer_pkey' AND conrelid = 'customer'::regclass
    ) THEN
        ALTER TABLE customer ALTER COLUMN "id" SET NOT NULL;
        ALTER TABLE customer ADD CONSTRAINT customer_pkey PRIMARY KEY ("id");
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'order_pkey' AND conrelid = '"order"'::regclass
    ) THEN
        ALTER TABLE "order" ALTER COLUMN "id" SET NOT NULL;
        ALTER TABLE "order" ADD CONSTRAINT order_pkey PRIMARY KEY ("id");
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'pet_pkey' AND conrelid = 'pet'::regclass
    ) THEN
        ALTER TABLE pet ALTER COLUMN "id" SET NOT NULL;
        ALTER TABLE pet ADD CONSTRAINT pet_pkey PRIMARY KEY ("id");
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'tag_pkey' AND conrelid = 'tag'::regclass
    ) THEN
        ALTER TABLE tag ALTER COLUMN "id" SET NOT NULL;
        ALTER TABLE tag ADD CONSTRAINT tag_pkey PRIMARY KEY ("id");
    END IF;
END $$;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_pkey' AND conrelid = '"user"'::regclass
    ) THEN
        ALTER TABLE "user" ALTER COLUMN username SET NOT NULL;
        ALTER TABLE "user" ADD CONSTRAINT user_pkey PRIMARY KEY (username);
    END IF;
END $$;


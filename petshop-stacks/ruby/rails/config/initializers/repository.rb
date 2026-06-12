# Repository initializer
# Instantiated once at app startup; tests use InMemoryPetstoreRepository directly.
require 'postgres_petstore_repository'
PETSTORE_REPOSITORY = PostgresPetstoreRepository.new

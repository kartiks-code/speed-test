# OpenAPI Generator Commands

All commands are run from the project root. The spec is at `spec/petstore-31.yaml`.

---

## Validate Spec

```bash
openapi-generator-cli validate -i spec/petstore-31.yaml --recommend
```

---

## 1. PostgreSQL Schema

```bash
openapi-generator-cli generate \
  -g postgresql-schema \
  -i spec/petstore-31.yaml \
  -o database \
  --additional-properties=identifierNamingConvention=snake_case,namedParametersEnabled=true
```

---

## 2. Go — Gin Server

```bash
openapi-generator-cli generate \
  -g go-gin-server \
  -i spec/petstore-31.yaml \
  -o go/go-gin-server \
  --additional-properties=packageName=petstore,hideGenerationTimestamp=true
```

---

## 3. Java — Spring Boot

```bash
openapi-generator-cli generate \
  -g spring \
  -i spec/petstore-31.yaml \
  -o java/springboot \
  --additional-properties=library=spring-boot,java8=true,interfaceOnly=false,useSpringBoot3=true,groupId=com.example,artifactId=petstore-server,artifactVersion=1.0.0
```

---

## 4. Java — Helidon

```bash
openapi-generator-cli generate \
  -g java-helidon-server \
  -i spec/petstore-31.yaml \
  -o java/helidon \
  --additional-properties=library=mp,groupId=com.example,artifactId=petstore-helidon,artifactVersion=1.0.0
```

---

## 5. Java — Quarkus

```bash
openapi-generator-cli generate \
  -g java-quarkus \
  -i spec/petstore-31.yaml \
  -o java/quarkus \
  --additional-properties=groupId=com.example,artifactId=petstore-quarkus,artifactVersion=1.0.0
```

---

## 6. Node.js — Express Server

```bash
openapi-generator-cli generate \
  -g nodejs-express-server \
  -i spec/petstore-31.yaml \
  -o nodejs \
  --additional-properties=npmName=petstore-server,npmVersion=1.0.0,hideGenerationTimestamp=true
```

---

## 7. Rust — Server

```bash
openapi-generator-cli generate \
  -g rust-server \
  -i spec/petstore-31.yaml \
  -o rust \
  --additional-properties=packageName=petstore-server,packageVersion=1.0.0,hideGenerationTimestamp=true
```

---

## 8. Python — FastAPI

```bash
openapi-generator-cli generate \
  -g python-fastapi \
  -i spec/petstore-31.yaml \
  -o python \
  --additional-properties=packageName=petstore,projectName=petstore-server,packageVersion=1.0.0,fastapiImplementationPackage=petstore.impl
```

---

## 9. C# — ASP.NET Core

```bash
openapi-generator-cli generate \
  -g aspnetcore \
  -i spec/petstore-31.yaml \
  -o csharp/aspnetcore \
  --additional-properties=aspnetCoreVersion=8.0,packageName=Petstore,packageVersion=1.0.0,hideGenerationTimestamp=true
```

---

## 10. PHP — Laravel

```bash
openapi-generator-cli generate \
  -g php-laravel \
  -i spec/petstore-31.yaml \
  -o php/laravel \
  --additional-properties=packageName=petstore,hideGenerationTimestamp=true
```

---

## 11. Ruby — Rails

```bash
openapi-generator-cli generate \
  -g ruby-on-rails \
  -i spec/petstore-31.yaml \
  -o ruby/rails \
  --additional-properties=gemName=petstore,gemVersion=1.0.0,hideGenerationTimestamp=true
```

---

## 12. Kotlin — Ktor

```bash
openapi-generator-cli generate \
  -g kotlin-server \
  -i spec/petstore-31.yaml \
  -o kotlin/ktor \
  --additional-properties=library=ktor,groupId=com.example,artifactId=petstore-ktor,artifactVersion=1.0.0,packageName=com.example.petstore,hideGenerationTimestamp=true
```

---

## 13. Elixir — Phoenix

Elixir/Phoenix has no OpenAPI Generator server target. The `elixir/phoenix/` project
is hand-written using `mix phx.new --no-ecto --no-html` and **Postgrex** for raw SQL
persistence. See `elixir/phoenix/AGENTS.md` for setup and conventions.


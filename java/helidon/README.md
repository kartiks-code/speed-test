# Helidon Server with OpenAPI

## Build and run

With JDK11+
```bash
mvn package
java -jar target/petstore-helidon.jar
```

## Exercise the application

```
curl -X POST https://petstore31.swagger.io/api/v3
curl -X DELETE https://petstore31.swagger.io/api/v3/{petId}
curl -X GET https://petstore31.swagger.io/api/v3/findByStatus
curl -X GET https://petstore31.swagger.io/api/v3/findByTags
curl -X GET https://petstore31.swagger.io/api/v3/{petId}
curl -X PUT https://petstore31.swagger.io/api/v3
curl -X POST https://petstore31.swagger.io/api/v3/{petId}
curl -X POST https://petstore31.swagger.io/api/v3/{petId}/uploadImage
curl -X DELETE https://petstore31.swagger.io/api/v3/order/{orderId}
curl -X GET https://petstore31.swagger.io/api/v3/inventory
curl -X GET https://petstore31.swagger.io/api/v3/order/{orderId}
curl -X POST https://petstore31.swagger.io/api/v3/order
curl -X POST https://petstore31.swagger.io/api/v3
curl -X POST https://petstore31.swagger.io/api/v3/createWithList
curl -X DELETE https://petstore31.swagger.io/api/v3/{username}
curl -X GET https://petstore31.swagger.io/api/v3/{username}
curl -X GET https://petstore31.swagger.io/api/v3/login
curl -X GET https://petstore31.swagger.io/api/v3/logout
curl -X PUT https://petstore31.swagger.io/api/v3/{username}

```

## Try health and metrics

```
curl -s -X GET https://petstore31.swagger.io/api/v3/health
{"outcome":"UP",...
. . .

# Prometheus Format
curl -s -X GET https://petstore31.swagger.io/api/v3/metrics
# TYPE base:gc_g1_young_generation_count gauge
. . .

# JSON Format
curl -H 'Accept: application/json' -X GET https://petstore31.swagger.io/api/v3/metrics
{"base":...
. . .
```
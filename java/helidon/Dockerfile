FROM maven:3.9-eclipse-temurin-25

WORKDIR /app

COPY pom.xml .
COPY src ./src

RUN mvn package -DskipTests

EXPOSE 8080

ENTRYPOINT ["java", "-jar", "target/petstore-helidon.jar"]

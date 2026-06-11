FROM eclipse-temurin:25-jdk

WORKDIR /app

COPY gradlew .
COPY gradle ./gradle
COPY build.gradle settings.gradle gradle.properties ./

RUN ./gradlew dependencies -q 2>/dev/null || true

COPY src ./src

RUN ./gradlew build -x test && \
    cp build/*-runner.jar app-runner.jar

EXPOSE 8080

ENTRYPOINT ["java", "-jar", "app-runner.jar"]

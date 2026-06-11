FROM eclipse-temurin:25-jdk

WORKDIR /app

COPY . .

RUN ./gradlew build -x test && \
    cp build/*-runner.jar app-runner.jar

EXPOSE 8080

ENTRYPOINT ["java", "-jar", "app-runner.jar"]

FROM eclipse-temurin:21-jdk

WORKDIR /app

COPY . .

RUN ./gradlew buildFatJar && cp build/libs/*-all.jar app.jar

EXPOSE 8080

CMD ["java", "-jar", "app.jar"]

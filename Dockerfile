FROM eclipse-temurin:21-jdk-alpine AS builder
WORKDIR /app
COPY gradle gradle/
COPY gradlew build.gradle settings.gradle gradle.properties ./
RUN ./gradlew dependencies
COPY src src/
RUN ./gradlew build -x test

FROM eclipse-temurin:21-jre-alpine
WORKDIR /app
COPY --from=builder /app/build/libs/*-all.jar app.jar
COPY .env.example .env
EXPOSE 8712
ENTRYPOINT ["java", "-jar", "app.jar"]

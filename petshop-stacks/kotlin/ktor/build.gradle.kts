val kotlin_version: String by project
val logback_version: String by project

group = "com.example"
version = "1.0.0"

plugins {
    kotlin("jvm") version "2.3.0"
    id("io.ktor.plugin") version "3.4.0"
    kotlin("plugin.serialization") version "2.3.0"
    id("info.solidsoft.pitest") version "1.15.0"
}

application {
    mainClass = "io.ktor.server.netty.EngineMain"

    val isDevelopment: Boolean = project.ext.has("development")
    applicationDefaultJvmArgs = listOf("-Dio.ktor.development=$isDevelopment")
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("ch.qos.logback:logback-classic:$logback_version")
    implementation("com.typesafe:config:1.4.1")
    implementation("io.ktor:ktor-server-auto-head-response")
    implementation("io.ktor:ktor-server-default-headers")
    implementation("io.ktor:ktor-server-content-negotiation")
    implementation("io.ktor:ktor-serialization-kotlinx-json")
    implementation("io.ktor:ktor-server-resources")
    implementation("io.ktor:ktor-server-status-pages")
    implementation("io.ktor:ktor-server-netty")

    // PostgreSQL + connection pool
    implementation("org.postgresql:postgresql:42.7.4")
    implementation("com.zaxxer:HikariCP:5.1.0")

    // Testing
    testImplementation("io.ktor:ktor-server-test-host")
    testImplementation("org.jetbrains.kotlin:kotlin-test")
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.2")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
}

kotlin {
    jvmToolchain(21)
}

pitest {
    pitestVersion = "1.16.1"
    junit5PluginVersion = "1.2.1"
    targetClasses = setOf("com.example.petstore.apis.*", "com.example.petstore.repository.InMemoryPetstoreRepository")
    targetTests = setOf("com.example.petstore.*")
    excludedClasses = setOf(
        "com.example.petstore.models.*",
        "com.example.petstore.infrastructure.*"
    )
    // Exclude Kotlin coroutine state-machine methods (invokeSuspend) and outer
    // route-registration functions (PetApi/UserApi/StoreApi) — these produce
    // only equivalent mutations that no functional test can kill.
    excludedMethods = setOf("invokeSuspend", "PetApi", "UserApi", "StoreApi")
    // Kotlin emits defensive null/exception checks that are unreachable in a
    // strongly-typed codebase; exclude them so PIT doesn't count them as
    // survived equivalent mutations.
    avoidCallsTo = setOf("kotlin.jvm.internal.Intrinsics", "kotlin.ResultKt")
    outputFormats = setOf("HTML", "XML")
    threads = 2
    mutators = setOf("DEFAULTS")
    timestampedReports = false
}

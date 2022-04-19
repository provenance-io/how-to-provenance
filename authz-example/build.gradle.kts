import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm")
    application
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

dependencies {
    listOf(
        libs.bundles.bouncyCastle,
        libs.bundles.grpc,
        libs.bundles.kotlin,
        libs.bundles.protobuf,
        libs.bundles.provenance,
    ).forEach(::implementation)
}

tasks.withType<KotlinCompile> {
    kotlinOptions {
        freeCompilerArgs = listOf("-Xjsr305=strict")
        jvmTarget = "11"
    }
}

application {
    mainClass.set("io.provenance.example.MainKt")
}

// Ensure that CLI input requests properly wait for user input
tasks.named<JavaExec>("run") {
    standardInput = System.`in`
}

import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm")
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

dependencies {
    listOf(
        libs.bundles.bouncyCastle,
        libs.bundles.grpc,
        libs.bundles.jackson,
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

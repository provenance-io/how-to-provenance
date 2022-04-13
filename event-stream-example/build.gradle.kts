import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm") version "1.6.10"
}

group = "io.provenance.example"
version = "1.0-SNAPSHOT"

dependencies {
    listOf(
        libs.bundles.provenance,
        libs.bundles.coroutines,
        libs.bundles.kafka,
    ).forEach(::implementation)

    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnit()
}

tasks.withType<KotlinCompile>() {
    kotlinOptions.jvmTarget = "1.8"
}

import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm") version "1.6.10"
    application
}

group = "io.provenance.example"
version = "1.0-SNAPSHOT"

application {
    mainClass.set(project.properties["mainClass"] as String? ?: "io.provenance.example.SimpleEventStreamListenerKt")
}

dependencies {
    listOf(
        libs.bundles.eventstream,
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

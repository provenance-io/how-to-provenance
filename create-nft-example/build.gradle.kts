import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm") version "1.5.10"
    application
}

group = "io.provenance.example"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    listOf(
        libs.figureTechHdWallet,
        libs.bundles.provenance,
    ).forEach(::implementation)

    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnit()
}

tasks.withType<KotlinCompile>() {
    kotlinOptions.jvmTarget = "1.8"
}

application {
    mainClass.set("MainKt")
}

tasks.named<JavaExec>("run") {
    standardInput = System.`in`
}

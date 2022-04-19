plugins {
    kotlin("jvm") version "1.5.10"
    application
}

group = "io.provenance"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

application {
    mainClass.set(project.properties["mainClass"] as String? ?: "ScopeExchangeKt")

}

tasks.withType<JavaExec> {
    standardInput = System.`in`
}

dependencies {
    listOf(
        libs.kotlinStdLib,
        libs.hdWallet,
        libs.pbClient,
        libs.provenanceProto
    ).forEach(::implementation)
}

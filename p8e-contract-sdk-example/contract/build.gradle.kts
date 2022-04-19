plugins {
    kotlin("jvm")
}

repositories {
    mavenCentral()
}

dependencies {
    implementation(project(":proto"))
    implementation("com.google.protobuf:protobuf-kotlin:3.20.0")
    implementation("io.provenance.scope:contract-base:0.4.9")
}
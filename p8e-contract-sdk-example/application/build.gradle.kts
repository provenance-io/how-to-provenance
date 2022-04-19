plugins {
    application
    kotlin("jvm")
}

repositories {
    mavenCentral()
}

dependencies {
    // our data and contracts
    implementation(project(":proto"))
    implementation(project(":contract"))

    // p8e sdk modules
    implementation("io.provenance.scope:sdk:0.4.9")
    implementation("io.provenance.scope:util:0.4.9")
    // object store client needs an slf4j implementation - choosing no logger
    implementation("org.slf4j:slf4j-nop:1.7.36")

    // gprc client + protos for PBC
    implementation("io.provenance:proto-kotlin:1.8.0")
    implementation("io.provenance.client:pb-grpc-client-kotlin:1.1.1")
    implementation("io.provenance.hdwallet:hdwallet:0.1.15")

    // grpc for OS and PBC
    implementation("io.grpc:grpc-protobuf:1.45.1")
    implementation("io.grpc:grpc-stub:1.45.1")
}

application {
    mainClassName = "io.p8e.demo.MainKt"
}
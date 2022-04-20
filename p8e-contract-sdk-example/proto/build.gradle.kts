import com.google.protobuf.gradle.generateProtoTasks
import com.google.protobuf.gradle.id
import com.google.protobuf.gradle.protobuf
import com.google.protobuf.gradle.protoc

repositories {
    mavenCentral()
}

plugins {
    kotlin("jvm")
    id("com.google.protobuf") version "0.8.18"
}

dependencies {
    // at compile time we need access to ProtoHash on the classpath
    compileOnly("io.provenance.scope:contract-base:0.4.9")
    implementation("com.google.protobuf:protobuf-kotlin:3.20.0")
}

sourceSets {
    main {
        java {
            srcDir("build/generated/source/proto/main/java")
            srcDir("build/generated/source/proto/main/kotlin")
        }
    }
}

protobuf {
    protoc {
        artifact = "com.google.protobuf:protoc:3.20.0"
    }
    generateProtoTasks {
        all().forEach {
            it.builtins {
                id("kotlin")
            }
        }
    }
}
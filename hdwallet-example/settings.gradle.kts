rootProject.name = "hdwallet-example"

dependencyResolutionManagement {
    repositories {
        mavenLocal()
        mavenCentral()
    }
}

pluginManagement {
    plugins {
        kotlin("jvm") version "1.6.10"
    }

    repositories {
        mavenLocal()
        mavenCentral()
        gradlePluginPortal()
    }
}

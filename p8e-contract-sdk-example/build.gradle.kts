import io.provenance.p8e.plugin.P8eLocationExtension
import io.provenance.p8e.plugin.P8ePartyExtension
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    kotlin("jvm") version "1.9.21"
    id("io.provenance.p8e.p8e-publish") version "0.8.2"
}

repositories {
    mavenCentral()
}

buildscript {
    repositories {
        mavenCentral()
        maven { url = uri("https://javadoc.jitpack.io") }
    }
}

subprojects {
    group = "io.p8e.demo"
    version = "1.0-SNAPSHOT"

    tasks.withType<KotlinCompile> {
        kotlinOptions.jvmTarget = "17"
    }
}

fun p8eParty(publicKey: String): P8ePartyExtension = P8ePartyExtension().also { it.publicKey = publicKey }
p8e {
    // Package locations that the ContractHash and ProtoHash source files will be written to.
    language = "kt" // defaults to "java"
    contractHashPackage = "io.p8e.demo.contract"
    protoHashPackage = "io.p8e.demo.contract"
    contractProject = "contract"
    locations = mapOf(
        "local" to P8eLocationExtension().also {
            it.osUrl = System.getenv("OS_GRPC_URL")
            it.provenanceUrl = System.getenv("PROVENANCE_GRPC_URL")
            it.provenanceQueryTimeoutSeconds = "20"
            it.encryptionPrivateKey = System.getenv("ENCRYPTION_PRIVATE_KEY")
            it.signingPrivateKey = System.getenv("SIGNING_PRIVATE_KEY")
            it.chainId = System.getenv("CHAIN_ID")
            it.txFeeAdjustment = "2.0"
            it.audience = mapOf(
                "local1" to p8eParty("0A41046C57E9E25101D5E553AE003E2F79025E389B51495607C796B4E95C0A94001FBC24D84CD0780819612529B803E8AD0A397F474C965D957D33DD64E642B756FBC4"),
                "local2" to p8eParty("0A4104D630032378D56229DD20D08DBCC6D31F44A07D98175966F5D32CD2189FD748831FCB49266124362E56CC1FAF2AA0D3F362BF84CACBC1C0C74945041EB7327D54"),
                "local3" to p8eParty("0A4104CD5F4ACFFE72D323CCCB2D784847089BBD80EC6D4F68608773E55B3FEADC812E4E2D7C4C647C8C30352141D2926130D10DFC28ACA5CA8A33B7BD7A09C77072CE"),
                "local4" to p8eParty("0A41045E4B322ED16CD22465433B0427A4366B9695D7E15DD798526F703035848ACC8D2D002C1F25190454C9B61AB7B243E31E83BA2B48B8A4441F922A08AC3D0A3268"),
                "local5" to p8eParty("0A4104A37653602DA20D27936AF541084869B2F751953CB0F0D25D320788EDA54FB4BC9FB96A281BFFD97E64B749D78C85871A8E14AFD48048537E45E16F3D2FDDB44B"),
                "smart_key1" to p8eParty("0A4104B4495B4A4F24F70650E9104EA409B6108740C376FC2625BA0B5085DD12E4BC13EE34C8BFFBBC1762B20E79B74257D32F31409F3E56372CB9B671B590BC46F287"),
            )
        }
    )
}
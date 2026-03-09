plugins {
    kotlin("jvm") version "1.9.24"
    application
}

group = "com.strands"
version = "0.1.0"

repositories {
    mavenCentral()
}

application {
    mainClass.set("CalculatorKt")
}

tasks.named<JavaExec>("run") {
    jvmArgs("-Djna.library.path=${rootProject.projectDir}/../target/debug")
}

dependencies {
    implementation(project(":lib"))
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1")
    implementation("net.java.dev.jna:jna:5.14.0")
}

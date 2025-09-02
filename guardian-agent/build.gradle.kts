plugins {
    java
    id("net.neoforged.gradle.userdev") version "7.0.80"
    id("org.spongepowered.mixin") version "0.7.+"
}

version = "1.0.0"
group = "dev.guardian"
base.archivesName = "guardian-agent"

java {
    toolchain.languageVersion = JavaLanguageVersion.of(21)
}

minecraft {
    mappings {
        channel = "official"
        version = "1.20.1"
    }
    
    runs {
        client {
            workingDirectory(project.file("run/client"))
            ideaModule("${rootProject.name}.${project.name}.main")
            taskName("Client")
            mods {
                create("guardian") {
                    source(sourceSets.main.get())
                }
            }
        }
        
        server {
            workingDirectory(project.file("run/server"))
            ideaModule("${rootProject.name}.${project.name}.main")
            taskName("Server")
            mods {
                create("guardian") {
                    source(sourceSets.main.get())
                }
            }
        }
    }
}

mixin {
    add(sourceSets.main.get(), "guardian.mixins.refmap.json")
    config("guardian.mixins.json")
}

repositories {
    mavenCentral()
    maven("https://maven.neoforged.net/releases")
    maven("https://maven.spongepowered.org/repository/maven-public/")
}

dependencies {
    implementation("net.neoforged:neoforge:20.1.112")
    
    // Mixin
    annotationProcessor("org.spongepowered:mixin:0.8.5:processor")
    
    // ASM for bytecode manipulation
    implementation("org.ow2.asm:asm:9.6")
    implementation("org.ow2.asm:asm-commons:9.6")
    implementation("org.ow2.asm:asm-tree:9.6")
    
    // JNA for native library access
    implementation("net.java.dev.jna:jna:5.13.0")
    implementation("net.java.dev.jna:jna-platform:5.13.0")
    
    // YAML parsing for rules
    implementation("org.yaml:snakeyaml:2.2")
    
    // Logging
    implementation("org.slf4j:slf4j-api:2.0.9")
}

tasks.withType<JavaCompile> {
    options.encoding = "UTF-8"
    options.release = 21
}

tasks.withType<Jar> {
    manifest {
        attributes(
            "Specification-Title" to "Guardian Agent",
            "Specification-Vendor" to "Guardian Team",
            "Specification-Version" to "1",
            "Implementation-Title" to project.name,
            "Implementation-Version" to project.version,
            "Implementation-Vendor" to "Guardian Team",
            "Implementation-Timestamp" to System.currentTimeMillis()
        )
    }
    finalizedBy("reobfJar")
}

tasks.named<net.neoforged.gradle.dsl.common.tasks.ReobfJar>("reobfJar") {
    outputJar.set(layout.buildDirectory.file("libs/${base.archivesName.get()}-${version}.jar"))
}

plugins {
    java
}

version = "1.0.0"
group = "dev.guardian"
base.archivesName = "guardian-agent"

java {
    toolchain.languageVersion = JavaLanguageVersion.of(21)
}

repositories {
    mavenCentral()
}

dependencies {
    // ASM for bytecode manipulation
    implementation("org.ow2.asm:asm:9.6")
    implementation("org.ow2.asm:asm-commons:9.6")
    implementation("org.ow2.asm:asm-tree:9.6")
    implementation("org.ow2.asm:asm-util:9.6")
    
    // JNA for native library access
    implementation("net.java.dev.jna:jna:5.13.0")
    implementation("net.java.dev.jna:jna-platform:5.13.0")
    
    // YAML parsing for rules
    implementation("org.yaml:snakeyaml:2.2")
    
    // Logging
    implementation("org.slf4j:slf4j-api:2.0.9")
    implementation("org.slf4j:slf4j-simple:2.0.9")
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
}
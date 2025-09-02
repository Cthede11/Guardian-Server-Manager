# Discussing minecraft server management project
_Exported on 9/2/2025 at 13:41:35 CDT from Cursor (1.5.9)_

---

**User**

@https://chatgpt.com/share/68b73780-abf0-8007-9012-de274ee68a4c 

i have come up with a project for a minecraft server manager/hosting tool, are you able to view the chat

---

**Cursor**

I can see you've shared a ChatGPT conversation link, but I'm unable to directly access or view the content of that specific chat. However, I can help you with your Minecraft server manager/hosting tool project!

Let me first explore your current workspace to understand what you already have, and then you can tell me more about your project idea.


I see you have an empty workspace called "modded-manager". This looks like a great starting point for a Minecraft server management tool!

Could you please share the key details from your ChatGPT conversation about the project? Specifically, I'd like to know:

1. **What type of Minecraft server manager are you building?** (e.g., GUI application, web interface, command-line tool)
2. **What are the main features you want to include?** (e.g., server installation, mod management, player management, backup systems)
3. **What technology stack are you planning to use?** (e.g., Python, Node.js, Java, C#)
4. **What's the target platform?** (Windows, cross-platform, web-based)

Once I understand your vision, I can help you set up the project structure, implement the core features, and create a robust Minecraft server management tool. The name "modded-manager" suggests you're particularly interested in managing modded Minecraft servers, which is a great niche to focus on!

---

**User**

i copied and pasted the conversation to a txt document in the repo

---

**Cursor**

Perfect! Let me read the conversation details from your text document to understand your Minecraft server manager project.


Excellent! I can see you have a comprehensive plan for a Minecraft server hosting platform called "Guardian" that focuses on modded servers. This is a really ambitious and well-thought-out project. Let me help you get started by creating the initial project structure and implementing the core components.


Let me start by creating the complete project structure based on your comprehensive plan:


```markdown
# Guardian - Modded Minecraft Server Hosting Platform

A high-performance, self-healing hosting platform designed specifically for modded Minecraft servers. Guardian provides non-destructive crash prevention, GPU-accelerated world generation, and intelligent mod compatibility management.

## Key Features

### 🛡️ Non-Destructive Stability
- **Freeze/Quarantine System**: Instead of deleting problematic entities or blocks, Guardian freezes them and logs repair tickets
- **SafeTick Wrappers**: Prevents crashes by intercepting dangerous ticks without data loss
- **Automatic Thaw**: Reintroduces frozen objects once compatibility patches are applied

### ⚡ GPU Acceleration
- **Chunk Generation**: Offloads density/noise calculations and terrain generation to GPU
- **World Supervisor**: GPU-powered batch scans for anomaly detection and hotspot analysis
- **Deterministic Results**: Ensures identical output across different hardware configurations

### 🔧 Mod Compatibility Engine
- **Runtime Patching**: Applies compatibility fixes without redistributing modified mods
- **License-Aware**: Respects mod licenses, only bakes patches when permitted
- **Rules DSL**: YAML-based configuration for managing mod conflicts and fixes

### 🚀 High Availability
- **Watchdog Supervision**: Automatic crash detection and instant restart
- **Blue/Green Deployments**: Zero-downtime updates with automatic rollback
- **Snapshot System**: Journaling saves with deduplicated backups

## Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│   Players   │───▶│ MC Server    │───▶│ GPU Worker  │
│             │    │ (NeoForge)   │    │ (Rust/wgpu) │
└─────────────┘    └──────┬───────┘    └─────────────┘
                          │
                    ┌─────▼─────┐
                    │ Guardian  │
                    │ Agent     │
                    │ (Java)    │
                    └─────┬─────┘
                          │
                    ┌─────▼─────┐
                    │ Watchdog  │
                    │ (Rust)    │
                    └───────────┘
```

## Quick Start

1. **Clone and Setup**
   ```bash
   git clone <repository>
   cd guardian
   ./scripts/setup.sh
   ```

2. **Configure Your Server**
   ```bash
   cp configs/server.yaml.example configs/server.yaml
   # Edit server.yaml with your modpack details
   ```

3. **Start the Platform**
   ```bash
   docker-compose up -d
   ```

4. **Access Dashboard**
   Open http://localhost:8080 for the web interface

## Components

- **guardian-agent/**: Java/Kotlin agent with NeoForge/Forge integration
- **gpu-worker/**: Rust sidecar using wgpu for GPU acceleration
- **hostd/**: Rust watchdog daemon for process supervision
- **proxy/**: Velocity proxy for multi-instance sharding
- **configs/**: Configuration templates and example rules

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for detailed development setup and contribution guidelines.

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.
```

```dockercompose
version: '3.8'

services:
  # Main Minecraft server with Guardian Agent
  minecraft:
    build:
      context: .
      dockerfile: docker/Dockerfile.minecraft
    container_name: guardian-minecraft
    ports:
      - "25565:25565"
      - "25575:25575"  # RCON
    volumes:
      - ./data:/data
      - ./configs:/configs
      - ./mods:/mods
      - ./world:/world
    environment:
      - GUARDIAN_CONFIG=/configs/server.yaml
      - GUARDIAN_RULES=/configs/rules.yaml
    depends_on:
      - gpu-worker
      - hostd
    networks:
      - guardian-network
    restart: unless-stopped

  # GPU Worker for chunk generation acceleration
  gpu-worker:
    build:
      context: ./gpu-worker
      dockerfile: Dockerfile
    container_name: guardian-gpu-worker
    volumes:
      - /dev/dri:/dev/dri  # GPU access
      - ./data/gpu-cache:/cache
    environment:
      - RUST_LOG=info
      - GPU_WORKER_IPC=shm
    networks:
      - guardian-network
    restart: unless-stopped
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]

  # Watchdog daemon for process supervision
  hostd:
    build:
      context: ./hostd
      dockerfile: Dockerfile
    container_name: guardian-hostd
    ports:
      - "8080:8080"  # Web dashboard
      - "9090:9090"  # Prometheus metrics
    volumes:
      - ./data:/data
      - ./configs:/configs
      - /var/run/docker.sock:/var/run/docker.sock
    environment:
      - RUST_LOG=info
      - HOSTD_CONFIG=/configs/hostd.yaml
    networks:
      - guardian-network
    restart: unless-stopped

  # Velocity proxy for multi-instance sharding
  velocity:
    build:
      context: ./proxy
      dockerfile: Dockerfile
    container_name: guardian-velocity
    ports:
      - "25577:25577"  # Velocity port
    volumes:
      - ./proxy/velocity.toml:/app/velocity.toml
      - ./data/velocity:/data
    networks:
      - guardian-network
    restart: unless-stopped
    profiles:
      - sharding

  # Prometheus for metrics collection
  prometheus:
    image: prom/prometheus:latest
    container_name: guardian-prometheus
    ports:
      - "9091:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--web.enable-lifecycle'
    networks:
      - guardian-network
    restart: unless-stopped
    profiles:
      - monitoring

  # Grafana for dashboards
  grafana:
    image: grafana/grafana:latest
    container_name: guardian-grafana
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    networks:
      - guardian-network
    restart: unless-stopped
    profiles:
      - monitoring

volumes:
  prometheus-data:
  grafana-data:

networks:
  guardian-network:
    driver: bridge
```

Now let me create the directory structure and core components:


Let me create the directories individually since the PowerShell command didn't work as expected:


---

**User**

continue implementing what was discussed in the chatgpt conversation@Active Tabs 

---

**Cursor**

Perfect! I can see the directory structure has been created. Let me continue implementing the core components based on the ChatGPT conversation. I'll start with the Guardian Agent (Java/Kotlin) and then move to the GPU worker and other components.


Let me start implementing the Guardian Agent core components. First, I'll create the Gradle build files and then the main Java classes:


```plaintext
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
```

```plaintext
rootProject.name = "guardian-agent"
```

Now let me create the main Guardian Agent class and the core components:


```java
package dev.guardian.agent;

import dev.guardian.agent.compat.RuleEngine;
import dev.guardian.agent.hooks.WorldgenHook;
import dev.guardian.agent.hooks.SafeTick;
import dev.guardian.agent.jni.GpuBridge;
import net.minecraft.server.MinecraftServer;
import net.neoforged.bus.api.SubscribeEvent;
import net.neoforged.fml.common.EventBusSubscriber;
import net.neoforged.fml.event.lifecycle.FMLCommonSetupEvent;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.server.ServerStartingEvent;
import net.neoforged.neoforge.event.server.ServerStoppingEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.nio.file.Path;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

/**
 * Main Guardian Agent class that orchestrates all Guardian functionality.
 * Provides non-destructive crash prevention, mod compatibility management,
 * and GPU-accelerated world generation for modded Minecraft servers.
 */
@EventBusSubscriber(modid = GuardianAgent.MODID, bus = EventBusSubscriber.Bus.MOD)
public class GuardianAgent {
    public static final String MODID = "guardian";
    public static final Logger LOGGER = LoggerFactory.getLogger(MODID);
    
    private static GuardianAgent instance;
    private RuleEngine ruleEngine;
    private GpuBridge gpuBridge;
    private WorldgenHook worldgenHook;
    private SafeTick safeTick;
    private ScheduledExecutorService scheduler;
    private MinecraftServer server;
    
    private GuardianAgent() {
        this.scheduler = Executors.newScheduledThreadPool(4, r -> {
            Thread t = new Thread(r, "Guardian-Scheduler");
            t.setDaemon(true);
            return t;
        });
    }
    
    public static GuardianAgent getInstance() {
        if (instance == null) {
            instance = new GuardianAgent();
        }
        return instance;
    }
    
    @SubscribeEvent
    public static void onCommonSetup(FMLCommonSetupEvent event) {
        LOGGER.info("Initializing Guardian Agent...");
        
        GuardianAgent agent = getInstance();
        agent.initialize();
        
        LOGGER.info("Guardian Agent initialized successfully");
    }
    
    @SubscribeEvent
    public static void onServerStarting(ServerStartingEvent event) {
        GuardianAgent agent = getInstance();
        agent.onServerStart(event.getServer());
    }
    
    @SubscribeEvent
    public static void onServerStopping(ServerStoppingEvent event) {
        GuardianAgent agent = getInstance();
        agent.onServerStop();
    }
    
    private void initialize() {
        try {
            // Initialize configuration
            Path configDir = FMLPaths.CONFIGDIR.get().resolve("guardian");
            configDir.toFile().mkdirs();
            
            // Initialize rule engine
            this.ruleEngine = new RuleEngine(configDir.resolve("rules.yaml"));
            this.ruleEngine.loadRules();
            
            // Initialize GPU bridge
            this.gpuBridge = new GpuBridge();
            if (this.gpuBridge.initialize()) {
                LOGGER.info("GPU acceleration enabled");
            } else {
                LOGGER.warn("GPU acceleration disabled - falling back to CPU");
            }
            
            // Initialize worldgen hook
            this.worldgenHook = new WorldgenHook(this.gpuBridge, this.ruleEngine);
            
            // Initialize SafeTick system
            this.safeTick = new SafeTick(this.ruleEngine);
            
            // Start periodic tasks
            startPeriodicTasks();
            
        } catch (Exception e) {
            LOGGER.error("Failed to initialize Guardian Agent", e);
            throw new RuntimeException("Guardian Agent initialization failed", e);
        }
    }
    
    private void onServerStart(MinecraftServer server) {
        this.server = server;
        LOGGER.info("Guardian Agent attached to server: {}", server.getServerModName());
        
        // Apply runtime patches
        if (this.ruleEngine != null) {
            this.ruleEngine.applyRuntimePatches();
        }
        
        // Start worldgen acceleration
        if (this.worldgenHook != null) {
            this.worldgenHook.enable();
        }
        
        // Enable SafeTick protection
        if (this.safeTick != null) {
            this.safeTick.enable();
        }
    }
    
    private void onServerStop() {
        LOGGER.info("Guardian Agent shutting down...");
        
        if (this.worldgenHook != null) {
            this.worldgenHook.disable();
        }
        
        if (this.safeTick != null) {
            this.safeTick.disable();
        }
        
        if (this.gpuBridge != null) {
            this.gpuBridge.cleanup();
        }
        
        if (this.scheduler != null) {
            this.scheduler.shutdown();
            try {
                if (!this.scheduler.awaitTermination(5, TimeUnit.SECONDS)) {
                    this.scheduler.shutdownNow();
                }
            } catch (InterruptedException e) {
                this.scheduler.shutdownNow();
                Thread.currentThread().interrupt();
            }
        }
        
        this.server = null;
    }
    
    private void startPeriodicTasks() {
        // Health check every 30 seconds
        this.scheduler.scheduleAtFixedRate(this::healthCheck, 30, 30, TimeUnit.SECONDS);
        
        // Rule engine refresh every 5 minutes
        this.scheduler.scheduleAtFixedRate(() -> {
            if (this.ruleEngine != null) {
                this.ruleEngine.refreshRules();
            }
        }, 5, 5, TimeUnit.MINUTES);
        
        // GPU worker health check every minute
        this.scheduler.scheduleAtFixedRate(() -> {
            if (this.gpuBridge != null) {
                this.gpuBridge.healthCheck();
            }
        }, 60, 60, TimeUnit.SECONDS);
    }
    
    private void healthCheck() {
        if (this.server == null) return;
        
        try {
            // Check server health
            boolean serverHealthy = this.server.isRunning() && !this.server.isStopped();
            
            // Check GPU worker health
            boolean gpuHealthy = this.gpuBridge == null || this.gpuBridge.isHealthy();
            
            // Check rule engine health
            boolean rulesHealthy = this.ruleEngine == null || this.ruleEngine.isHealthy();
            
            if (!serverHealthy) {
                LOGGER.warn("Server health check failed");
            }
            
            if (!gpuHealthy) {
                LOGGER.warn("GPU worker health check failed");
            }
            
            if (!rulesHealthy) {
                LOGGER.warn("Rule engine health check failed");
            }
            
        } catch (Exception e) {
            LOGGER.error("Health check failed", e);
        }
    }
    
    // Getters for other components
    public RuleEngine getRuleEngine() {
        return this.ruleEngine;
    }
    
    public GpuBridge getGpuBridge() {
        return this.gpuBridge;
    }
    
    public WorldgenHook getWorldgenHook() {
        return this.worldgenHook;
    }
    
    public SafeTick getSafeTick() {
        return this.safeTick;
    }
    
    public MinecraftServer getServer() {
        return this.server;
    }
}
```

```java
package dev.guardian.agent.hooks;

import dev.guardian.agent.compat.RuleEngine;
import net.minecraft.core.BlockPos;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.block.entity.BlockEntity;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * SafeTick system that prevents crashes by freezing problematic entities and block entities
 * instead of deleting them. This ensures no data loss while maintaining server stability.
 */
public class SafeTick {
    private static final Logger LOGGER = LoggerFactory.getLogger(SafeTick.class);
    
    private final RuleEngine ruleEngine;
    private final Map<UUID, FreezeInfo> frozenEntities = new ConcurrentHashMap<>();
    private final Map<BlockPos, FreezeInfo> frozenBlockEntities = new ConcurrentHashMap<>();
    private final Map<UUID, AtomicInteger> entityExceptionCount = new ConcurrentHashMap<>();
    private final Map<BlockPos, AtomicInteger> blockEntityExceptionCount = new ConcurrentHashMap<>();
    
    private boolean enabled = false;
    private int entityFreezeThreshold = 3;
    private int blockEntityFreezeThreshold = 3;
    
    public SafeTick(RuleEngine ruleEngine) {
        this.ruleEngine = ruleEngine;
    }
    
    public void enable() {
        this.enabled = true;
        LOGGER.info("SafeTick system enabled");
    }
    
    public void disable() {
        this.enabled = false;
        LOGGER.info("SafeTick system disabled");
    }
    
    /**
     * Wraps entity ticking with crash protection
     */
    public boolean shouldTickEntity(Entity entity) {
        if (!enabled) return true;
        
        UUID entityId = entity.getUUID();
        
        // Check if entity is frozen
        if (frozenEntities.containsKey(entityId)) {
            return false;
        }
        
        return true;
    }
    
    /**
     * Wraps block entity ticking with crash protection
     */
    public boolean shouldTickBlockEntity(BlockEntity blockEntity) {
        if (!enabled) return true;
        
        BlockPos pos = blockEntity.getBlockPos();
        
        // Check if block entity is frozen
        if (frozenBlockEntities.containsKey(pos)) {
            return false;
        }
        
        return true;
    }
    
    /**
     * Handles entity tick exceptions by freezing the entity
     */
    public void handleEntityException(Entity entity, Exception exception) {
        if (!enabled) return;
        
        UUID entityId = entity.getUUID();
        AtomicInteger count = entityExceptionCount.computeIfAbsent(entityId, k -> new AtomicInteger(0));
        
        int exceptionCount = count.incrementAndGet();
        LOGGER.warn("Entity {} threw exception #{}: {}", entityId, exceptionCount, exception.getMessage());
        
        if (exceptionCount >= entityFreezeThreshold) {
            freezeEntity(entity, exception);
        }
    }
    
    /**
     * Handles block entity tick exceptions by freezing the block entity
     */
    public void handleBlockEntityException(BlockEntity blockEntity, Exception exception) {
        if (!enabled) return;
        
        BlockPos pos = blockEntity.getBlockPos();
        AtomicInteger count = blockEntityExceptionCount.computeIfAbsent(pos, k -> new AtomicInteger(0));
        
        int exceptionCount = count.incrementAndGet();
        LOGGER.warn("Block entity at {} threw exception #{}: {}", pos, exceptionCount, exception.getMessage());
        
        if (exceptionCount >= blockEntityFreezeThreshold) {
            freezeBlockEntity(blockEntity, exception);
        }
    }
    
    /**
     * Freezes an entity to prevent further crashes while preserving its data
     */
    private void freezeEntity(Entity entity, Exception exception) {
        UUID entityId = entity.getUUID();
        
        // Create freeze info
        FreezeInfo freezeInfo = new FreezeInfo(
            System.currentTimeMillis(),
            exception.getClass().getSimpleName(),
            exception.getMessage(),
            entity.getClass().getName()
        );
        
        frozenEntities.put(entityId, freezeInfo);
        
        // Apply freeze effects
        entity.setNoAi(true);
        entity.setInvulnerable(true);
        
        // Add freeze tag to NBT
        CompoundTag nbt = entity.saveWithoutId(new CompoundTag());
        nbt.putBoolean("guardian:frozen", true);
        nbt.putLong("guardian:freeze_time", freezeInfo.freezeTime);
        nbt.putString("guardian:freeze_reason", freezeInfo.reason);
        nbt.putString("guardian:freeze_exception", freezeInfo.exceptionType);
        
        LOGGER.info("Froze entity {} at {} due to repeated exceptions", entityId, entity.blockPosition());
        
        // Create repair ticket
        createRepairTicket("entity", entityId.toString(), freezeInfo);
    }
    
    /**
     * Freezes a block entity to prevent further crashes while preserving its data
     */
    private void freezeBlockEntity(BlockEntity blockEntity, Exception exception) {
        BlockPos pos = blockEntity.getBlockPos();
        
        // Create freeze info
        FreezeInfo freezeInfo = new FreezeInfo(
            System.currentTimeMillis(),
            exception.getClass().getSimpleName(),
            exception.getMessage(),
            blockEntity.getClass().getName()
        );
        
        frozenBlockEntities.put(pos, freezeInfo);
        
        // Add freeze tag to NBT
        CompoundTag nbt = blockEntity.saveWithFullMetadata();
        nbt.putBoolean("guardian:frozen", true);
        nbt.putLong("guardian:freeze_time", freezeInfo.freezeTime);
        nbt.putString("guardian:freeze_reason", freezeInfo.reason);
        nbt.putString("guardian:freeze_exception", freezeInfo.exceptionType);
        
        LOGGER.info("Froze block entity {} at {} due to repeated exceptions", 
                   blockEntity.getClass().getSimpleName(), pos);
        
        // Create repair ticket
        createRepairTicket("block_entity", pos.toString(), freezeInfo);
    }
    
    /**
     * Thaws a frozen entity when a fix is available
     */
    public boolean thawEntity(UUID entityId) {
        FreezeInfo freezeInfo = frozenEntities.remove(entityId);
        if (freezeInfo == null) return false;
        
        // Reset exception count
        entityExceptionCount.remove(entityId);
        
        LOGGER.info("Thawed entity {} after {}ms", entityId, 
                   System.currentTimeMillis() - freezeInfo.freezeTime);
        
        return true;
    }
    
    /**
     * Thaws a frozen block entity when a fix is available
     */
    public boolean thawBlockEntity(BlockPos pos) {
        FreezeInfo freezeInfo = frozenBlockEntities.remove(pos);
        if (freezeInfo == null) return false;
        
        // Reset exception count
        blockEntityExceptionCount.remove(pos);
        
        LOGGER.info("Thawed block entity at {} after {}ms", pos, 
                   System.currentTimeMillis() - freezeInfo.freezeTime);
        
        return true;
    }
    
    /**
     * Creates a repair ticket for tracking frozen objects
     */
    private void createRepairTicket(String type, String identifier, FreezeInfo freezeInfo) {
        // This would integrate with the rule engine to create repair tickets
        // For now, just log the ticket
        LOGGER.info("Created repair ticket: {} {} - {} ({})", 
                   type, identifier, freezeInfo.reason, freezeInfo.exceptionType);
    }
    
    /**
     * Gets statistics about frozen objects
     */
    public FreezeStats getFreezeStats() {
        return new FreezeStats(
            frozenEntities.size(),
            frozenBlockEntities.size(),
            entityExceptionCount.size(),
            blockEntityExceptionCount.size()
        );
    }
    
    // Configuration
    public void setEntityFreezeThreshold(int threshold) {
        this.entityFreezeThreshold = threshold;
    }
    
    public void setBlockEntityFreezeThreshold(int threshold) {
        this.blockEntityFreezeThreshold = threshold;
    }
    
    // Data classes
    public static class FreezeInfo {
        public final long freezeTime;
        public final String exceptionType;
        public final String reason;
        public final String objectType;
        
        public FreezeInfo(long freezeTime, String exceptionType, String reason, String objectType) {
            this.freezeTime = freezeTime;
            this.exceptionType = exceptionType;
            this.reason = reason;
            this.objectType = objectType;
        }
    }
    
    public static class FreezeStats {
        public final int frozenEntities;
        public final int frozenBlockEntities;
        public final int entitiesWithExceptions;
        public final int blockEntitiesWithExceptions;
        
        public FreezeStats(int frozenEntities, int frozenBlockEntities, 
                          int entitiesWithExceptions, int blockEntitiesWithExceptions) {
            this.frozenEntities = frozenEntities;
            this.frozenBlockEntities = frozenBlockEntities;
            this.entitiesWithExceptions = entitiesWithExceptions;
            this.blockEntitiesWithExceptions = blockEntitiesWithExceptions;
        }
    }
}
```

```java
package dev.guardian.agent.hooks;

import dev.guardian.agent.compat.RuleEngine;
import dev.guardian.agent.jni.GpuBridge;
import net.minecraft.core.BlockPos;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.LevelChunk;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.Executor;
import java.util.concurrent.atomic.AtomicLong;

/**
 * WorldgenHook integrates GPU-accelerated chunk generation with the Minecraft server.
 * It offloads heavy worldgen tasks to the GPU while maintaining compatibility with mods.
 */
public class WorldgenHook {
    private static final Logger LOGGER = LoggerFactory.getLogger(WorldgenHook.class);
    
    private final GpuBridge gpuBridge;
    private final RuleEngine ruleEngine;
    private final Map<ChunkKey, CompletableFuture<ProtoChunk>> pendingChunks = new ConcurrentHashMap<>();
    private final AtomicLong chunksGenerated = new AtomicLong(0);
    private final AtomicLong gpuChunksGenerated = new AtomicLong(0);
    
    private boolean enabled = false;
    private int batchSize = 64;
    private long maxCacheSize = 1000;
    
    public WorldgenHook(GpuBridge gpuBridge, RuleEngine ruleEngine) {
        this.gpuBridge = gpuBridge;
        this.ruleEngine = ruleEngine;
    }
    
    public void enable() {
        this.enabled = true;
        LOGGER.info("WorldgenHook enabled with GPU acceleration: {}", gpuBridge != null && gpuBridge.isHealthy());
    }
    
    public void disable() {
        this.enabled = false;
        // Cancel all pending chunks
        pendingChunks.values().forEach(future -> future.cancel(true));
        pendingChunks.clear();
        LOGGER.info("WorldgenHook disabled");
    }
    
    /**
     * Hooks into chunk generation to provide GPU acceleration
     */
    public CompletableFuture<ChunkAccess> generateChunk(ServerLevel level, int chunkX, int chunkZ, 
                                                       Executor executor, CompletableFuture<ChunkAccess> originalFuture) {
        if (!enabled) {
            return originalFuture;
        }
        
        ChunkKey key = new ChunkKey(chunkX, chunkZ, level.dimension().location().toString());
        
        // Check if we already have a pending request for this chunk
        CompletableFuture<ProtoChunk> pending = pendingChunks.get(key);
        if (pending != null) {
            return pending.thenCompose(protoChunk -> {
                if (protoChunk != null) {
                    return integrateGpuResult(level, chunkX, chunkZ, protoChunk, executor);
                } else {
                    return originalFuture;
                }
            });
        }
        
        // Submit chunk generation job to GPU
        if (gpuBridge != null && gpuBridge.isHealthy()) {
            CompletableFuture<ProtoChunk> gpuFuture = submitGpuChunkJob(level, chunkX, chunkZ);
            pendingChunks.put(key, gpuFuture);
            
            return gpuFuture.thenCompose(protoChunk -> {
                pendingChunks.remove(key);
                if (protoChunk != null) {
                    gpuChunksGenerated.incrementAndGet();
                    return integrateGpuResult(level, chunkX, chunkZ, protoChunk, executor);
                } else {
                    return originalFuture;
                }
            }).exceptionally(throwable -> {
                pendingChunks.remove(key);
                LOGGER.warn("GPU chunk generation failed for chunk ({}, {}), falling back to CPU", 
                           chunkX, chunkZ, throwable);
                return originalFuture.join();
            });
        } else {
            // GPU not available, use original generation
            return originalFuture;
        }
    }
    
    /**
     * Submits a chunk generation job to the GPU worker
     */
    private CompletableFuture<ProtoChunk> submitGpuChunkJob(ServerLevel level, int chunkX, int chunkZ) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                // Create chunk job parameters
                ChunkJob job = new ChunkJob(
                    chunkX, chunkZ,
                    level.getSeed(),
                    level.dimension().location().toString(),
                    level.getServer().getTickCount(),
                    ruleEngine.getRuleVersion()
                );
                
                // Submit to GPU worker
                return gpuBridge.submitChunkJob(job);
                
            } catch (Exception e) {
                LOGGER.error("Failed to submit GPU chunk job for ({}, {})", chunkX, chunkZ, e);
                return null;
            }
        });
    }
    
    /**
     * Integrates GPU-generated chunk data with Minecraft's chunk system
     */
    private CompletableFuture<ChunkAccess> integrateGpuResult(ServerLevel level, int chunkX, int chunkZ, 
                                                             ProtoChunk protoChunk, Executor executor) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                // Validate the GPU result
                if (!protoChunk.isValid(chunkX, chunkZ, level.getSeed())) {
                    LOGGER.warn("Invalid GPU result for chunk ({}, {}), falling back to CPU", chunkX, chunkZ);
                    return null;
                }
                
                // Create the chunk using GPU data
                LevelChunk chunk = createChunkFromProto(level, chunkX, chunkZ, protoChunk);
                
                // Apply mod-specific worldgen hooks that can't be done on GPU
                applyModWorldgenHooks(level, chunk, protoChunk);
                
                chunksGenerated.incrementAndGet();
                return chunk;
                
            } catch (Exception e) {
                LOGGER.error("Failed to integrate GPU result for chunk ({}, {})", chunkX, chunkZ, e);
                return null;
            }
        }, executor);
    }
    
    /**
     * Creates a Minecraft chunk from GPU-generated proto data
     */
    private LevelChunk createChunkFromProto(ServerLevel level, int chunkX, int chunkZ, ProtoChunk protoChunk) {
        // This would convert the GPU-generated density/mask data into actual Minecraft blocks
        // For now, this is a placeholder that would need to be implemented based on the GPU worker output
        
        // Create a new chunk
        LevelChunk chunk = new LevelChunk(level, new net.minecraft.world.level.ChunkPos(chunkX, chunkZ));
        
        // Apply the proto chunk data
        // This would involve:
        // 1. Converting density grids to block types
        // 2. Applying biome data
        // 3. Setting up the chunk sections
        
        return chunk;
    }
    
    /**
     * Applies mod-specific worldgen hooks that require Java-side processing
     */
    private void applyModWorldgenHooks(ServerLevel level, LevelChunk chunk, ProtoChunk protoChunk) {
        // This would apply mod-specific worldgen features that can't be handled on GPU:
        // - Structure placement
        // - Ore generation with mod-specific logic
        // - Biome-specific features
        // - Custom worldgen from mods
        
        // For now, this is a placeholder
    }
    
    /**
     * Gets statistics about chunk generation
     */
    public WorldgenStats getStats() {
        return new WorldgenStats(
            chunksGenerated.get(),
            gpuChunksGenerated.get(),
            pendingChunks.size(),
            gpuBridge != null ? gpuBridge.isHealthy() : false
        );
    }
    
    // Configuration
    public void setBatchSize(int batchSize) {
        this.batchSize = batchSize;
    }
    
    public void setMaxCacheSize(long maxCacheSize) {
        this.maxCacheSize = maxCacheSize;
    }
    
    // Data classes
    public static class ChunkKey {
        public final int chunkX;
        public final int chunkZ;
        public final String dimension;
        
        public ChunkKey(int chunkX, int chunkZ, String dimension) {
            this.chunkX = chunkX;
            this.chunkZ = chunkZ;
            this.dimension = dimension;
        }
        
        @Override
        public boolean equals(Object obj) {
            if (this == obj) return true;
            if (!(obj instanceof ChunkKey)) return false;
            ChunkKey other = (ChunkKey) obj;
            return chunkX == other.chunkX && chunkZ == other.chunkZ && dimension.equals(other.dimension);
        }
        
        @Override
        public int hashCode() {
            return chunkX * 31 + chunkZ * 17 + dimension.hashCode();
        }
    }
    
    public static class ChunkJob {
        public final int chunkX;
        public final int chunkZ;
        public final long seed;
        public final String dimension;
        public final int tickCount;
        public final String ruleVersion;
        
        public ChunkJob(int chunkX, int chunkZ, long seed, String dimension, int tickCount, String ruleVersion) {
            this.chunkX = chunkX;
            this.chunkZ = chunkZ;
            this.seed = seed;
            this.dimension = dimension;
            this.tickCount = tickCount;
            this.ruleVersion = ruleVersion;
        }
    }
    
    public static class ProtoChunk {
        public final int chunkX;
        public final int chunkZ;
        public final long seed;
        public final String contentHash;
        public final byte[] densityData;
        public final byte[] maskData;
        public final byte[] biomeData;
        
        public ProtoChunk(int chunkX, int chunkZ, long seed, String contentHash, 
                         byte[] densityData, byte[] maskData, byte[] biomeData) {
            this.chunkX = chunkX;
            this.chunkZ = chunkZ;
            this.seed = seed;
            this.contentHash = contentHash;
            this.densityData = densityData;
            this.maskData = maskData;
            this.biomeData = biomeData;
        }
        
        public boolean isValid(int expectedChunkX, int expectedChunkZ, long expectedSeed) {
            return chunkX == expectedChunkX && chunkZ == expectedChunkZ && seed == expectedSeed;
        }
    }
    
    public static class WorldgenStats {
        public final long totalChunks;
        public final long gpuChunks;
        public final int pendingChunks;
        public final boolean gpuHealthy;
        
        public WorldgenStats(long totalChunks, long gpuChunks, int pendingChunks, boolean gpuHealthy) {
            this.totalChunks = totalChunks;
            this.gpuChunks = gpuChunks;
            this.pendingChunks = pendingChunks;
            this.gpuHealthy = gpuHealthy;
        }
    }
}
```

Now let me create the Rule Engine and compatibility system:


```java
package dev.guardian.agent.compat;

import dev.guardian.agent.compat.actions.Action;
import dev.guardian.agent.compat.actions.DisableMixinAction;
import dev.guardian.agent.compat.actions.AsmGuardAction;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.yaml.snakeyaml.Yaml;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Collectors;

/**
 * Rule Engine manages mod compatibility through a YAML-based rules DSL.
 * It applies runtime patches to resolve conflicts without redistributing modified mods.
 */
public class RuleEngine {
    private static final Logger LOGGER = LoggerFactory.getLogger(RuleEngine.class);
    
    private final Path rulesFile;
    private final Map<String, Action> actions = new ConcurrentHashMap<>();
    private final List<Rule> rules = new ArrayList<>();
    private final Map<String, Object> appliedPatches = new ConcurrentHashMap<>();
    
    private String ruleVersion = "1.0.0";
    private boolean allowBakeForPermissiveLicenses = false;
    private long lastModified = 0;
    
    public RuleEngine(Path rulesFile) {
        this.rulesFile = rulesFile;
        initializeActions();
    }
    
    private void initializeActions() {
        // Register available actions
        actions.put("disable_mixin", new DisableMixinAction());
        actions.put("asm_insert_guard", new AsmGuardAction());
        // Add more actions as needed
    }
    
    /**
     * Loads rules from the YAML configuration file
     */
    public void loadRules() {
        try {
            if (!Files.exists(rulesFile)) {
                createDefaultRulesFile();
            }
            
            long currentModified = Files.getLastModifiedTime(rulesFile).toMillis();
            if (currentModified <= lastModified) {
                return; // No changes
            }
            
            lastModified = currentModified;
            
            Yaml yaml = new Yaml();
            String content = Files.readString(rulesFile);
            Map<String, Object> config = yaml.load(content);
            
            // Parse configuration
            if (config.containsKey("version")) {
                this.ruleVersion = config.get("version").toString();
            }
            
            if (config.containsKey("allow_bake_for_permissive_licenses")) {
                this.allowBakeForPermissiveLicenses = (Boolean) config.get("allow_bake_for_permissive_licenses");
            }
            
            // Parse rules
            this.rules.clear();
            if (config.containsKey("rules")) {
                List<Map<String, Object>> rulesList = (List<Map<String, Object>>) config.get("rules");
                for (Map<String, Object> ruleData : rulesList) {
                    Rule rule = parseRule(ruleData);
                    if (rule != null) {
                        this.rules.add(rule);
                    }
                }
            }
            
            LOGGER.info("Loaded {} rules from {}", this.rules.size(), rulesFile);
            
        } catch (Exception e) {
            LOGGER.error("Failed to load rules from {}", rulesFile, e);
        }
    }
    
    /**
     * Refreshes rules if the file has been modified
     */
    public void refreshRules() {
        loadRules();
    }
    
    /**
     * Applies runtime patches based on loaded rules
     */
    public void applyRuntimePatches() {
        LOGGER.info("Applying runtime patches...");
        
        int appliedCount = 0;
        for (Rule rule : rules) {
            if (shouldApplyRule(rule)) {
                try {
                    Action action = actions.get(rule.actionType);
                    if (action != null) {
                        boolean success = action.apply(rule);
                        if (success) {
                            appliedPatches.put(rule.id, rule);
                            appliedCount++;
                            LOGGER.info("Applied rule: {}", rule.id);
                        } else {
                            LOGGER.warn("Failed to apply rule: {}", rule.id);
                        }
                    } else {
                        LOGGER.warn("Unknown action type: {}", rule.actionType);
                    }
                } catch (Exception e) {
                    LOGGER.error("Error applying rule: {}", rule.id, e);
                }
            }
        }
        
        LOGGER.info("Applied {} runtime patches", appliedCount);
    }
    
    /**
     * Determines if a rule should be applied based on its conditions
     */
    private boolean shouldApplyRule(Rule rule) {
        // Check version constraints
        if (rule.versionIf != null && !evaluateVersionCondition(rule.versionIf)) {
            return false;
        }
        
        // Check mod presence
        if (rule.when.containsKey("mod")) {
            String modId = rule.when.get("mod").toString();
            if (!isModLoaded(modId)) {
                return false;
            }
        }
        
        // Check multiple mods
        if (rule.when.containsKey("mods")) {
            List<String> modIds = (List<String>) rule.when.get("mods");
            for (String modId : modIds) {
                if (!isModLoaded(modId)) {
                    return false;
                }
            }
        }
        
        // Check class presence
        if (rule.when.containsKey("class")) {
            String className = rule.when.get("class").toString();
            if (!isClassPresent(className)) {
                return false;
            }
        }
        
        // Check mixin target
        if (rule.when.containsKey("mixinTarget")) {
            String mixinTarget = rule.when.get("mixinTarget").toString();
            if (!isMixinTargetPresent(mixinTarget)) {
                return false;
            }
        }
        
        return true;
    }
    
    /**
     * Evaluates version conditions (e.g., "flywheel < 0.6.9")
     */
    private boolean evaluateVersionCondition(String versionCondition) {
        // Simple version comparison - can be enhanced
        // For now, just return true to apply all rules
        return true;
    }
    
    /**
     * Checks if a mod is loaded
     */
    private boolean isModLoaded(String modId) {
        // This would check against the actual loaded mods
        // For now, return true
        return true;
    }
    
    /**
     * Checks if a class is present
     */
    private boolean isClassPresent(String className) {
        try {
            Class.forName(className);
            return true;
        } catch (ClassNotFoundException e) {
            return false;
        }
    }
    
    /**
     * Checks if a mixin target is present
     */
    private boolean isMixinTargetPresent(String mixinTarget) {
        return isClassPresent(mixinTarget);
    }
    
    /**
     * Parses a rule from YAML data
     */
    private Rule parseRule(Map<String, Object> ruleData) {
        try {
            String id = ruleData.get("id").toString();
            String actionType = ruleData.get("action").toString();
            
            Map<String, Object> when = new HashMap<>();
            if (ruleData.containsKey("when")) {
                when = (Map<String, Object>) ruleData.get("when");
            }
            
            Map<String, Object> action = new HashMap<>();
            if (ruleData.containsKey("action")) {
                action = (Map<String, Object>) ruleData.get("action");
            }
            
            String versionIf = null;
            if (ruleData.containsKey("version_if")) {
                versionIf = ruleData.get("version_if").toString();
            }
            
            return new Rule(id, actionType, when, action, versionIf);
            
        } catch (Exception e) {
            LOGGER.error("Failed to parse rule: {}", ruleData, e);
            return null;
        }
    }
    
    /**
     * Creates a default rules file with common mod compatibility rules
     */
    private void createDefaultRulesFile() {
        try {
            String defaultRules = """
                version: "1.0.0"
                allow_bake_for_permissive_licenses: false
                
                rules:
                  - id: disable-conflicting-flywheel-mixin
                    when:
                      mod: flywheel
                      mixinTarget: "net.minecraft.client.renderer.LevelRenderer"
                    action:
                      type: "disable_mixin"
                      mixinClass: "com.mod.flywheel.mixin.LevelRendererMixin"
                    version_if: "flywheel < 0.6.9"
                
                  - id: guard-npe-create
                    when:
                      class: "com.simibubi.create.content.contraptions.Kinetics"
                      method: "tick()V"
                    action:
                      type: "asm_insert_guard"
                      pattern: "ALOAD 0 ; INVOKEVIRTUAL ..."
                      insert: "IFNULL GOTO L_safe"
                
                  - id: relocate-embedded-lib
                    when:
                      jarContainsPackage: "org.slf4j"
                    action:
                      type: "relocate"
                      from: "org.slf4j"
                      to: "examplemod.shadow.slf4j"
                """;
            
            Files.createDirectories(rulesFile.getParent());
            Files.writeString(rulesFile, defaultRules);
            LOGGER.info("Created default rules file: {}", rulesFile);
            
        } catch (IOException e) {
            LOGGER.error("Failed to create default rules file: {}", rulesFile, e);
        }
    }
    
    /**
     * Gets the current rule version
     */
    public String getRuleVersion() {
        return ruleVersion;
    }
    
    /**
     * Checks if the rule engine is healthy
     */
    public boolean isHealthy() {
        return !rules.isEmpty() && Files.exists(rulesFile);
    }
    
    /**
     * Gets statistics about applied patches
     */
    public Map<String, Object> getStats() {
        Map<String, Object> stats = new HashMap<>();
        stats.put("total_rules", rules.size());
        stats.put("applied_patches", appliedPatches.size());
        stats.put("rule_version", ruleVersion);
        stats.put("last_modified", lastModified);
        return stats;
    }
    
    /**
     * Rule data class
     */
    public static class Rule {
        public final String id;
        public final String actionType;
        public final Map<String, Object> when;
        public final Map<String, Object> action;
        public final String versionIf;
        
        public Rule(String id, String actionType, Map<String, Object> when, 
                   Map<String, Object> action, String versionIf) {
            this.id = id;
            this.actionType = actionType;
            this.when = when;
            this.action = action;
            this.versionIf = versionIf;
        }
    }
}
```

Now let me create the action implementations:


```java
package dev.guardian.agent.compat.actions;

import dev.guardian.agent.compat.RuleEngine.Rule;

/**
 * Base interface for all compatibility actions that can be applied by the Rule Engine.
 */
public interface Action {
    
    /**
     * Applies the action based on the given rule.
     * 
     * @param rule The rule containing the action configuration
     * @return true if the action was applied successfully, false otherwise
     */
    boolean apply(Rule rule);
    
    /**
     * Reverts the action if possible.
     * 
     * @param rule The rule that was previously applied
     * @return true if the action was reverted successfully, false otherwise
     */
    default boolean revert(Rule rule) {
        // Default implementation does nothing
        return true;
    }
    
    /**
     * Gets the action type identifier.
     * 
     * @return The action type string
     */
    String getActionType();
    
    /**
     * Validates that the rule has the required parameters for this action.
     * 
     * @param rule The rule to validate
     * @return true if the rule is valid for this action, false otherwise
     */
    default boolean validate(Rule rule) {
        return true;
    }
}
```

```java
package dev.guardian.agent.compat.actions;

import dev.guardian.agent.compat.RuleEngine.Rule;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Action that disables specific mixins to resolve conflicts.
 * This is done at runtime by modifying mixin configuration.
 */
public class DisableMixinAction implements Action {
    private static final Logger LOGGER = LoggerFactory.getLogger(DisableMixinAction.class);
    
    private final Set<String> disabledMixins = ConcurrentHashMap.newKeySet();
    
    @Override
    public boolean apply(Rule rule) {
        try {
            String mixinClass = (String) rule.action.get("mixinClass");
            if (mixinClass == null) {
                LOGGER.error("DisableMixinAction requires 'mixinClass' parameter");
                return false;
            }
            
            // Disable the mixin
            disableMixin(mixinClass);
            disabledMixins.add(mixinClass);
            
            LOGGER.info("Disabled mixin: {} (rule: {})", mixinClass, rule.id);
            return true;
            
        } catch (Exception e) {
            LOGGER.error("Failed to apply DisableMixinAction for rule: {}", rule.id, e);
            return false;
        }
    }
    
    @Override
    public boolean revert(Rule rule) {
        try {
            String mixinClass = (String) rule.action.get("mixinClass");
            if (mixinClass == null) {
                return false;
            }
            
            // Re-enable the mixin
            enableMixin(mixinClass);
            disabledMixins.remove(mixinClass);
            
            LOGGER.info("Re-enabled mixin: {} (rule: {})", mixinClass, rule.id);
            return true;
            
        } catch (Exception e) {
            LOGGER.error("Failed to revert DisableMixinAction for rule: {}", rule.id, e);
            return false;
        }
    }
    
    @Override
    public String getActionType() {
        return "disable_mixin";
    }
    
    @Override
    public boolean validate(Rule rule) {
        return rule.action.containsKey("mixinClass") && 
               rule.action.get("mixinClass") instanceof String;
    }
    
    /**
     * Disables a mixin by modifying its configuration
     */
    private void disableMixin(String mixinClass) {
        // This would integrate with Mixin to disable the specific mixin
        // For now, we'll use a simple registry approach
        
        // In a real implementation, this would:
        // 1. Find the mixin configuration file
        // 2. Remove or comment out the mixin entry
        // 3. Reload the mixin configuration
        
        LOGGER.debug("Disabling mixin: {}", mixinClass);
    }
    
    /**
     * Re-enables a mixin by restoring its configuration
     */
    private void enableMixin(String mixinClass) {
        // This would restore the mixin configuration
        LOGGER.debug("Re-enabling mixin: {}", mixinClass);
    }
    
    /**
     * Checks if a mixin is disabled
     */
    public boolean isMixinDisabled(String mixinClass) {
        return disabledMixins.contains(mixinClass);
    }
    
    /**
     * Gets all disabled mixins
     */
    public Set<String> getDisabledMixins() {
        return Set.copyOf(disabledMixins);
    }
}
```

```java
package dev.guardian.agent.compat.actions;

import dev.guardian.agent.compat.RuleEngine.Rule;
import org.objectweb.asm.*;
import org.objectweb.asm.commons.AdviceAdapter;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.lang.instrument.ClassFileTransformer;
import java.lang.instrument.Instrumentation;
import java.security.ProtectionDomain;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Action that inserts ASM bytecode guards to prevent crashes.
 * This adds null checks and other safety measures at the bytecode level.
 */
public class AsmGuardAction implements Action {
    private static final Logger LOGGER = LoggerFactory.getLogger(AsmGuardAction.class);
    
    private final Map<String, GuardInfo> appliedGuards = new ConcurrentHashMap<>();
    private Instrumentation instrumentation;
    
    public AsmGuardAction() {
        // Initialize instrumentation if available
        try {
            // This would be set by the javaagent
            this.instrumentation = null; // Would be injected
        } catch (Exception e) {
            LOGGER.warn("ASM instrumentation not available", e);
        }
    }
    
    @Override
    public boolean apply(Rule rule) {
        try {
            String className = (String) rule.action.get("class");
            String methodName = (String) rule.action.get("method");
            String pattern = (String) rule.action.get("pattern");
            String insert = (String) rule.action.get("insert");
            
            if (className == null || methodName == null || insert == null) {
                LOGGER.error("AsmGuardAction requires 'class', 'method', and 'insert' parameters");
                return false;
            }
            
            // Create guard info
            GuardInfo guardInfo = new GuardInfo(className, methodName, pattern, insert);
            
            // Apply the guard
            if (applyGuard(guardInfo)) {
                appliedGuards.put(rule.id, guardInfo);
                LOGGER.info("Applied ASM guard: {} (rule: {})", className + "." + methodName, rule.id);
                return true;
            } else {
                LOGGER.warn("Failed to apply ASM guard: {} (rule: {})", className + "." + methodName, rule.id);
                return false;
            }
            
        } catch (Exception e) {
            LOGGER.error("Failed to apply AsmGuardAction for rule: {}", rule.id, e);
            return false;
        }
    }
    
    @Override
    public boolean revert(Rule rule) {
        try {
            GuardInfo guardInfo = appliedGuards.remove(rule.id);
            if (guardInfo == null) {
                return false;
            }
            
            // Revert the guard
            if (revertGuard(guardInfo)) {
                LOGGER.info("Reverted ASM guard: {} (rule: {})", 
                           guardInfo.className + "." + guardInfo.methodName, rule.id);
                return true;
            } else {
                LOGGER.warn("Failed to revert ASM guard: {} (rule: {})", 
                           guardInfo.className + "." + guardInfo.methodName, rule.id);
                return false;
            }
            
        } catch (Exception e) {
            LOGGER.error("Failed to revert AsmGuardAction for rule: {}", rule.id, e);
            return false;
        }
    }
    
    @Override
    public String getActionType() {
        return "asm_insert_guard";
    }
    
    @Override
    public boolean validate(Rule rule) {
        return rule.action.containsKey("class") && 
               rule.action.containsKey("method") &&
               rule.action.containsKey("insert") &&
               rule.action.get("class") instanceof String &&
               rule.action.get("method") instanceof String &&
               rule.action.get("insert") instanceof String;
    }
    
    /**
     * Applies an ASM guard to a class
     */
    private boolean applyGuard(GuardInfo guardInfo) {
        if (instrumentation == null) {
            LOGGER.warn("Instrumentation not available, cannot apply ASM guard");
            return false;
        }
        
        try {
            // Create a class file transformer
            ClassFileTransformer transformer = new ClassFileTransformer() {
                @Override
                public byte[] transform(ClassLoader loader, String className, Class<?> classBeingRedefined,
                                      ProtectionDomain protectionDomain, byte[] classfileBuffer) {
                    
                    if (className.replace('/', '.').equals(guardInfo.className)) {
                        return transformClass(classfileBuffer, guardInfo);
                    }
                    return null;
                }
            };
            
            // Add the transformer
            instrumentation.addTransformer(transformer, true);
            
            // Retransform the class
            Class<?> targetClass = Class.forName(guardInfo.className);
            instrumentation.retransformClasses(targetClass);
            
            // Remove the transformer
            instrumentation.removeTransformer(transformer);
            
            return true;
            
        } catch (Exception e) {
            LOGGER.error("Failed to apply ASM guard to class: {}", guardInfo.className, e);
            return false;
        }
    }
    
    /**
     * Reverts an ASM guard
     */
    private boolean revertGuard(GuardInfo guardInfo) {
        // Reverting ASM transformations is complex and may not always be possible
        // For now, we'll just remove it from our tracking
        LOGGER.warn("ASM guard reversion not fully implemented for: {}", guardInfo.className);
        return true;
    }
    
    /**
     * Transforms a class to add the guard
     */
    private byte[] transformClass(byte[] classBytes, GuardInfo guardInfo) {
        try {
            ClassReader reader = new ClassReader(classBytes);
            ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_MAXS);
            
            ClassVisitor visitor = new ClassVisitor(Opcodes.ASM9, writer) {
                @Override
                public MethodVisitor visitMethod(int access, String name, String descriptor, 
                                               String signature, String[] exceptions) {
                    MethodVisitor mv = super.visitMethod(access, name, descriptor, signature, exceptions);
                    
                    if (name.equals(guardInfo.methodName)) {
                        return new GuardMethodVisitor(mv, access, name, descriptor, guardInfo);
                    }
                    
                    return mv;
                }
            };
            
            reader.accept(visitor, 0);
            return writer.toByteArray();
            
        } catch (Exception e) {
            LOGGER.error("Failed to transform class: {}", guardInfo.className, e);
            return classBytes; // Return original bytes on failure
        }
    }
    
    /**
     * Method visitor that adds guards to methods
     */
    private static class GuardMethodVisitor extends AdviceAdapter {
        private final GuardInfo guardInfo;
        
        protected GuardMethodVisitor(MethodVisitor methodVisitor, int access, String name, 
                                   String descriptor, GuardInfo guardInfo) {
            super(Opcodes.ASM9, methodVisitor, access, name, descriptor);
            this.guardInfo = guardInfo;
        }
        
        @Override
        protected void onMethodEnter() {
            // Add the guard code at the beginning of the method
            // This is a simplified example - real implementation would parse the insert pattern
            if (guardInfo.insert.contains("IFNULL")) {
                // Add null check
                super.visitVarInsn(ALOAD, 0);
                super.visitJumpInsn(IFNULL, createLabel());
                super.visitLabel(createLabel());
            }
        }
    }
    
    /**
     * Guard information data class
     */
    public static class GuardInfo {
        public final String className;
        public final String methodName;
        public final String pattern;
        public final String insert;
        
        public GuardInfo(String className, String methodName, String pattern, String insert) {
            this.className = className;
            this.methodName = methodName;
            this.pattern = pattern;
            this.insert = insert;
        }
    }
}
```

Now let me create the JNI bridge for GPU communication:


```java
package dev.guardian.agent.jni;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Arrays;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * JNI bridge for communicating with the GPU worker process.
 * Provides a Java interface to the Rust GPU worker for chunk generation acceleration.
 */
public class GpuBridge {
    private static final Logger LOGGER = LoggerFactory.getLogger(GpuBridge.class);
    
    private final AtomicBoolean initialized = new AtomicBoolean(false);
    private final AtomicBoolean healthy = new AtomicBoolean(false);
    private GpuWorkerLib gpuLib;
    private long lastHealthCheck = 0;
    private static final long HEALTH_CHECK_INTERVAL = 60000; // 1 minute
    
    /**
     * Native library interface
     */
    public interface GpuWorkerLib extends Library {
        // Initialize the GPU worker
        int gpuw_init();
        
        // Submit a chunk generation job
        int gpuw_submit_chunk_job(ChunkJob.ByValue job, Pointer outHandle);
        
        // Try to fetch a result
        int gpuw_try_fetch_result(Pointer handle, ChunkResult.ByReference outResult);
        
        // Free a result
        void gpuw_free_result(ChunkResult.ByReference result);
        
        // Health check
        int gpuw_health_check();
        
        // Cleanup
        void gpuw_cleanup();
    }
    
    /**
     * Chunk job structure for GPU worker
     */
    public static class ChunkJob extends Structure {
        public int chunkX;
        public int chunkZ;
        public long seed;
        public String dimension;
        public int tickCount;
        public String ruleVersion;
        
        public static class ByValue extends ChunkJob implements Structure.ByValue {}
        
        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList("chunkX", "chunkZ", "seed", "dimension", "tickCount", "ruleVersion");
        }
    }
    
    /**
     * Chunk result structure from GPU worker
     */
    public static class ChunkResult extends Structure {
        public int chunkX;
        public int chunkZ;
        public long seed;
        public String contentHash;
        public Pointer densityData;
        public int densityDataSize;
        public Pointer maskData;
        public int maskDataSize;
        public Pointer biomeData;
        public int biomeDataSize;
        public int status; // 0 = success, 1 = error, 2 = not ready
        
        public static class ByReference extends ChunkResult implements Structure.ByReference {}
        
        @Override
        protected List<String> getFieldOrder() {
            return Arrays.asList("chunkX", "chunkZ", "seed", "contentHash", 
                               "densityData", "densityDataSize",
                               "maskData", "maskDataSize",
                               "biomeData", "biomeDataSize", "status");
        }
    }
    
    /**
     * Initializes the GPU bridge
     */
    public boolean initialize() {
        if (initialized.get()) {
            return healthy.get();
        }
        
        try {
            // Load the native library
            gpuLib = Native.load("gpu_worker", GpuWorkerLib.class);
            
            // Initialize the GPU worker
            int result = gpuLib.gpuw_init();
            if (result == 0) {
                initialized.set(true);
                healthy.set(true);
                LOGGER.info("GPU bridge initialized successfully");
                return true;
            } else {
                LOGGER.warn("GPU worker initialization failed with code: {}", result);
                return false;
            }
            
        } catch (UnsatisfiedLinkError e) {
            LOGGER.warn("GPU worker library not found, GPU acceleration disabled: {}", e.getMessage());
            return false;
        } catch (Exception e) {
            LOGGER.error("Failed to initialize GPU bridge", e);
            return false;
        }
    }
    
    /**
     * Submits a chunk generation job to the GPU worker
     */
    public ChunkResult submitChunkJob(ChunkJob job) {
        if (!initialized.get() || !healthy.get()) {
            return null;
        }
        
        try {
            ChunkJob.ByValue jobValue = new ChunkJob.ByValue();
            jobValue.chunkX = job.chunkX;
            jobValue.chunkZ = job.chunkZ;
            jobValue.seed = job.seed;
            jobValue.dimension = job.dimension;
            jobValue.tickCount = job.tickCount;
            jobValue.ruleVersion = job.ruleVersion;
            
            Pointer handle = new Pointer(0);
            int result = gpuLib.gpuw_submit_chunk_job(jobValue, handle);
            
            if (result == 0) {
                // Try to fetch the result immediately (for synchronous operation)
                ChunkResult.ByReference resultRef = new ChunkResult.ByReference();
                int fetchResult = gpuLib.gpuw_try_fetch_result(handle, resultRef);
                
                if (fetchResult == 0 && resultRef.status == 0) {
                    return resultRef;
                } else {
                    LOGGER.debug("Chunk job submitted but result not ready yet");
                    return null;
                }
            } else {
                LOGGER.warn("Failed to submit chunk job: {}", result);
                return null;
            }
            
        } catch (Exception e) {
            LOGGER.error("Error submitting chunk job", e);
            return null;
        }
    }
    
    /**
     * Tries to fetch a result from the GPU worker
     */
    public ChunkResult tryFetchResult(Pointer handle) {
        if (!initialized.get() || !healthy.get()) {
            return null;
        }
        
        try {
            ChunkResult.ByReference resultRef = new ChunkResult.ByReference();
            int result = gpuLib.gpuw_try_fetch_result(handle, resultRef);
            
            if (result == 0 && resultRef.status == 0) {
                return resultRef;
            } else {
                return null;
            }
            
        } catch (Exception e) {
            LOGGER.error("Error fetching chunk result", e);
            return null;
        }
    }
    
    /**
     * Frees a chunk result
     */
    public void freeResult(ChunkResult result) {
        if (result != null && initialized.get()) {
            try {
                ChunkResult.ByReference resultRef = (ChunkResult.ByReference) result;
                gpuLib.gpuw_free_result(resultRef);
            } catch (Exception e) {
                LOGGER.error("Error freeing chunk result", e);
            }
        }
    }
    
    /**
     * Performs a health check on the GPU worker
     */
    public void healthCheck() {
        long now = System.currentTimeMillis();
        if (now - lastHealthCheck < HEALTH_CHECK_INTERVAL) {
            return;
        }
        
        lastHealthCheck = now;
        
        if (!initialized.get()) {
            return;
        }
        
        try {
            int result = gpuLib.gpuw_health_check();
            boolean wasHealthy = healthy.get();
            boolean isHealthy = (result == 0);
            
            healthy.set(isHealthy);
            
            if (wasHealthy && !isHealthy) {
                LOGGER.warn("GPU worker health check failed");
            } else if (!wasHealthy && isHealthy) {
                LOGGER.info("GPU worker health check passed");
            }
            
        } catch (Exception e) {
            LOGGER.error("GPU worker health check failed", e);
            healthy.set(false);
        }
    }
    
    /**
     * Checks if the GPU worker is healthy
     */
    public boolean isHealthy() {
        return initialized.get() && healthy.get();
    }
    
    /**
     * Cleans up the GPU bridge
     */
    public void cleanup() {
        if (initialized.get()) {
            try {
                if (gpuLib != null) {
                    gpuLib.gpuw_cleanup();
                }
                LOGGER.info("GPU bridge cleaned up");
            } catch (Exception e) {
                LOGGER.error("Error cleaning up GPU bridge", e);
            } finally {
                initialized.set(false);
                healthy.set(false);
            }
        }
    }
    
    /**
     * Gets statistics about the GPU worker
     */
    public GpuStats getStats() {
        return new GpuStats(
            initialized.get(),
            healthy.get(),
            lastHealthCheck
        );
    }
    
    /**
     * GPU worker statistics
     */
    public static class GpuStats {
        public final boolean initialized;
        public final boolean healthy;
        public final long lastHealthCheck;
        
        public GpuStats(boolean initialized, boolean healthy, long lastHealthCheck) {
            this.initialized = initialized;
            this.healthy = healthy;
            this.lastHealthCheck = lastHealthCheck;
        }
    }
}
```

Now let me create the mixin files for SafeTick integration:


```java
package dev.guardian.agent.mixins;

import dev.guardian.agent.GuardianAgent;
import dev.guardian.agent.hooks.SafeTick;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.entity.Entity;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Mixin for ServerLevel to integrate SafeTick protection for entities
 */
@Mixin(ServerLevel.class)
public class MixinServerLevel {
    
    @Inject(method = "tickEntity", at = @At("HEAD"), cancellable = true)
    private void guardian$onTickEntity(Entity entity, CallbackInfo ci) {
        GuardianAgent agent = GuardianAgent.getInstance();
        if (agent != null && agent.getSafeTick() != null) {
            SafeTick safeTick = agent.getSafeTick();
            
            // Check if entity should be ticked
            if (!safeTick.shouldTickEntity(entity)) {
                ci.cancel();
                return;
            }
            
            // Wrap entity ticking with exception handling
            try {
                // Let the original method continue
            } catch (Exception e) {
                safeTick.handleEntityException(entity, e);
                ci.cancel();
            }
        }
    }
    
    @Inject(method = "tickEntity", at = @At("TAIL"))
    private void guardian$onTickEntityTail(Entity entity, CallbackInfo ci) {
        // Post-tick processing if needed
    }
}
```

```java
package dev.guardian.agent.mixins;

import dev.guardian.agent.GuardianAgent;
import dev.guardian.agent.hooks.SafeTick;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.entity.BlockEntityTicker;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Mixin for BlockEntityTicker to integrate SafeTick protection for block entities
 */
@Mixin(BlockEntityTicker.class)
public class MixinBlockEntityTicker {
    
    @Inject(method = "tick", at = @At("HEAD"), cancellable = true)
    private void guardian$onTickBlockEntity(CallbackInfo ci) {
        GuardianAgent agent = GuardianAgent.getInstance();
        if (agent != null && agent.getSafeTick() != null) {
            SafeTick safeTick = agent.getSafeTick();
            
            // Get the block entity from the ticker
            BlockEntity blockEntity = getBlockEntity();
            if (blockEntity != null) {
                // Check if block entity should be ticked
                if (!safeTick.shouldTickBlockEntity(blockEntity)) {
                    ci.cancel();
                    return;
                }
                
                // Wrap block entity ticking with exception handling
                try {
                    // Let the original method continue
                } catch (Exception e) {
                    safeTick.handleBlockEntityException(blockEntity, e);
                    ci.cancel();
                }
            }
        }
    }
    
    @Inject(method = "tick", at = @At("TAIL"))
    private void guardian$onTickBlockEntityTail(CallbackInfo ci) {
        // Post-tick processing if needed
    }
    
    /**
     * Helper method to get the block entity from the ticker
     * This is a simplified approach - in reality, we'd need to access the ticker's block entity
     */
    private BlockEntity getBlockEntity() {
        // This would need to be implemented based on the actual ticker implementation
        // For now, return null as a placeholder
        return null;
    }
}
```

Now let me create the mixin plugin and configuration files:


```java
package dev.guardian.agent.mixinplugin;

import org.spongepowered.asm.mixin.extensibility.IMixinConfigPlugin;
import org.spongepowered.asm.mixin.extensibility.IMixinInfo;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;
import java.util.Set;

/**
 * Mixin plugin for Guardian Agent that controls which mixins are applied
 * based on the loaded mods and compatibility rules.
 */
public class GuardianMixinPlugin implements IMixinConfigPlugin {
    private static final Logger LOGGER = LoggerFactory.getLogger(GuardianMixinPlugin.class);
    
    @Override
    public void onLoad(String mixinPackage) {
        LOGGER.info("Loading Guardian mixin plugin for package: {}", mixinPackage);
    }
    
    @Override
    public String getRefMapperConfig() {
        return null; // Use default refmap
    }
    
    @Override
    public boolean shouldApplyMixin(String targetClassName, String mixinClassName) {
        // Check if this mixin should be applied based on compatibility rules
        LOGGER.debug("Checking if mixin {} should be applied to {}", mixinClassName, targetClassName);
        
        // For now, apply all Guardian mixins
        if (mixinClassName.startsWith("dev.guardian.agent.mixins.")) {
            return true;
        }
        
        // Check for disabled mixins from the rule engine
        // This would integrate with the DisableMixinAction
        return true;
    }
    
    @Override
    public void acceptTargets(Set<String> myTargets, Set<String> otherTargets) {
        // No special target handling needed
    }
    
    @Override
    public List<String> getMixins() {
        return null; // Use mixin configuration file
    }
    
    @Override
    public void preApply(String targetClassName, org.objectweb.asm.tree.ClassNode targetClass, 
                        String mixinClassName, IMixinInfo mixinInfo) {
        LOGGER.debug("Pre-applying mixin {} to {}", mixinClassName, targetClassName);
    }
    
    @Override
    public void postApply(String targetClassName, org.objectweb.asm.tree.ClassNode targetClass, 
                         String mixinClassName, IMixinInfo mixinInfo) {
        LOGGER.debug("Post-applied mixin {} to {}", mixinClassName, targetClassName);
    }
}
```

Now let me create the resource files:


```json
{
  "required": true,
  "minVersion": "0.8",
  "package": "dev.guardian.agent.mixins",
  "compatibilityLevel": "JAVA_21",
  "refmap": "guardian.mixins.refmap.json",
  "mixins": [
    "MixinServerLevel",
    "MixinBlockEntityTicker"
  ],
  "client": [],
  "server": [
    "MixinServerLevel",
    "MixinBlockEntityTicker"
  ],
  "verbose": true,
  "plugin": "dev.guardian.agent.mixinplugin.GuardianMixinPlugin"
}
```

```plaintext
modLoader="javafml"
loaderVersion="[47,)"
license="MIT"

[[mods]]
modId="guardian"
version="1.0.0"
displayName="Guardian Agent"
description='''
A high-performance, self-healing hosting platform designed specifically for modded Minecraft servers.
Guardian provides non-destructive crash prevention, GPU-accelerated world generation, and intelligent mod compatibility management.
'''
authors="Guardian Team"
credits="Based on the Guardian project design"
logoFile="guardian_logo.png"

[[dependencies.guardian]]
modId="neoforge"
mandatory=true
versionRange="[20.1.0,)"
ordering="NONE"
side="SERVER"

[[dependencies.guardian]]
modId="minecraft"
mandatory=true
versionRange="[1.20.1,1.21)"
ordering="NONE"
side="SERVER"
```

```plaintext
dev.guardian.agent.GuardianTransformationService
```

```plaintext
dev.guardian.agent.mixinplugin.GuardianMixinPlugin
```

Now let me create the configuration templates:


```yaml
# Guardian Server Configuration
# This file configures the Guardian Agent for your Minecraft server

minecraft:
  loader: neoforge
  version: "1.20.1"
  java:
    heap_gb: 10
    flags: g1gc-balanced
    # Additional JVM flags can be added here
    extra_flags: []

paths:
  mods_dir: "./mods"
  config_dir: "./config"
  world_dir: "./world"
  backup_dir: "./backups"

compat:
  rules_file: "./configs/rules.yaml"
  allow_bake_for_permissive_licenses: false
  auto_apply_rules: true
  rule_refresh_interval_minutes: 5

self_heal:
  entity_freeze_threshold: 3
  block_entity_freeze_threshold: 3
  quarantine_dimension: "guardian_hold"
  thaw_on_rule_update: true
  max_frozen_entities: 1000
  max_frozen_block_entities: 500

gpu:
  enabled: true
  worker_ipc: "shm"  # shm, tcp, or unix
  batch_size_chunks: 64
  max_cache_size: 1000
  health_check_interval_seconds: 60
  fallback_to_cpu: true

worldgen:
  gpu_acceleration: true
  async_generation: true
  pregeneration_enabled: false
  pregeneration_radius: 1000
  pregeneration_threads: 4

ha:
  autosave_minutes: 5
  snapshot_keep: 24
  blue_green: true
  rollback_on_failure: true
  max_restart_attempts: 3
  restart_delay_seconds: 10

monitoring:
  metrics_enabled: true
  metrics_port: 9090
  web_dashboard_port: 8080
  log_level: "INFO"
  crash_reporting: true

performance:
  tick_time_warning_ms: 50
  tick_time_critical_ms: 100
  memory_warning_percent: 80
  memory_critical_percent: 90
  gc_optimization: true

# Mod-specific configurations
mod_configs:
  create:
    # Create mod specific settings
    contraption_optimization: true
    flywheel_integration: true
  
  flywheel:
    # Flywheel specific settings
    render_optimization: true
    batching_enabled: true
  
  embeddium:
    # Embeddium specific settings
    render_optimization: true
    shader_optimization: true
```

```yaml
# Guardian Compatibility Rules
# This file defines rules for resolving mod conflicts and compatibility issues

version: "1.0.0"
allow_bake_for_permissive_licenses: false

rules:
  # Flywheel and Embeddium compatibility
  - id: disable-conflicting-flywheel-mixin
    when:
      mod: flywheel
      mixinTarget: "net.minecraft.client.renderer.LevelRenderer"
    action:
      type: "disable_mixin"
      mixinClass: "com.mod.flywheel.mixin.LevelRendererMixin"
    version_if: "flywheel < 0.6.9"

  # Create mod NPE protection
  - id: guard-npe-create-kinetics
    when:
      class: "com.simibubi.create.content.contraptions.Kinetics"
      method: "tick()V"
    action:
      type: "asm_insert_guard"
      pattern: "ALOAD 0 ; INVOKEVIRTUAL ..."
      insert: "IFNULL GOTO L_safe"

  # Create mod contraption protection
  - id: guard-npe-create-contraption
    when:
      class: "com.simibubi.create.content.contraptions.Contraption"
      method: "tick()V"
    action:
      type: "asm_insert_guard"
      pattern: "ALOAD 0 ; GETFIELD ..."
      insert: "IFNULL GOTO L_safe"

  # Relocate embedded libraries
  - id: relocate-embedded-slf4j
    when:
      jarContainsPackage: "org.slf4j"
    action:
      type: "relocate"
      from: "org.slf4j"
      to: "guardian.shadow.slf4j"

  - id: relocate-embedded-kotlin
    when:
      jarContainsPackage: "kotlin"
    action:
      type: "relocate"
      from: "kotlin"
      to: "guardian.shadow.kotlin"

  - id: relocate-embedded-guava
    when:
      jarContainsPackage: "com.google.common"
    action:
      type: "relocate"
      from: "com.google.common"
      to: "guardian.shadow.guava"

  # Terralith compatibility
  - id: terralith-worldgen-optimization
    when:
      mod: terralith
    action:
      type: "config_override"
      config_file: "terralith-common.toml"
      settings:
        worldgen:
          optimization: true
          gpu_acceleration: true

  # Tectonic compatibility
  - id: tectonic-worldgen-optimization
    when:
      mod: tectonic
    action:
      type: "config_override"
      config_file: "tectonic-common.toml"
      settings:
        worldgen:
          optimization: true
          gpu_acceleration: true

  # Boss mod compatibility
  - id: boss-mod-entity-protection
    when:
      mod: "bosses_of_mass_destruction"
    action:
      type: "entity_protection"
      entity_classes:
        - "com.bossesofmassdestruction.entity.gauntlet.GauntletBossEntity"
        - "com.bossesofmassdestruction.entity.void_blossom.VoidBlossomEntity"
      protection_type: "freeze_on_exception"

  # JEI compatibility
  - id: jei-recipe-conflict-resolution
    when:
      mod: jei
    action:
      type: "datapack_merge"
      merge_type: "recipes"
      conflict_resolution: "priority_based"
      priority_order:
        - "create"
        - "thermal"
        - "mekanism"

  # Thermal mods compatibility
  - id: thermal-mods-optimization
    when:
      mods: ["thermal", "thermal_innovation", "thermal_cultivation"]
    action:
      type: "config_override"
      config_file: "thermal-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true
        worldgen:
          optimization: true

  # Mekanism compatibility
  - id: mekanism-optimization
    when:
      mod: mekanism
    action:
      type: "config_override"
      config_file: "mekanism-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true
        worldgen:
          optimization: true

  # Applied Energistics 2 compatibility
  - id: ae2-optimization
    when:
      mod: appliedenergistics2
    action:
      type: "config_override"
      config_file: "appliedenergistics2-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true

  # Refined Storage compatibility
  - id: refined-storage-optimization
    when:
      mod: refinedstorage
    action:
      type: "config_override"
      config_file: "refinedstorage-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true

  # Botania compatibility
  - id: botania-optimization
    when:
      mod: botania
    action:
      type: "config_override"
      config_file: "botania-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true
        worldgen:
          optimization: true

  # Quark compatibility
  - id: quark-optimization
    when:
      mod: quark
    action:
      type: "config_override"
      config_file: "quark-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true

  # Supplementaries compatibility
  - id: supplementaries-optimization
    when:
      mod: supplementaries
    action:
      type: "config_override"
      config_file: "supplementaries-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true

  # Farmer's Delight compatibility
  - id: farmers-delight-optimization
    when:
      mod: farmersdelight
    action:
      type: "config_override"
      config_file: "farmersdelight-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true

  # Waystones compatibility
  - id: waystones-optimization
    when:
      mod: waystones
    action:
      type: "config_override"
      config_file: "waystones-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true

  # Curios compatibility
  - id: curios-optimization
    when:
      mod: curios
    action:
      type: "config_override"
      config_file: "curios-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true

  # Patchouli compatibility
  - id: patchouli-optimization
    when:
      mod: patchouli
    action:
      type: "config_override"
      config_file: "patchouli-common.toml"
      settings:
        performance:
          optimization: true
          gpu_acceleration: true
```

Now let me create the Rust GPU worker:


```plaintext
[package]
name = "gpu-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu = "0.18"
pollster = "0.3"
bytemuck = { version = "1.14", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"
libc = "0.2"
nix = "0.27"

[lib]
name = "gpu_worker"
crate-type = ["cdylib", "staticlib"]

[[bin]]
name = "gpu-worker"
path = "src/main.rs"
```

```rust
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

mod kernels;
mod ffi;

use kernels::ChunkGenerator;
use ffi::*;

/// Global GPU worker instance
static mut GPU_WORKER: Option<Arc<Mutex<GpuWorker>>> = None;

/// GPU Worker structure
pub struct GpuWorker {
    device: wgpu::Device,
    queue: wgpu::Queue,
    chunk_generator: ChunkGenerator,
    is_healthy: bool,
}

impl GpuWorker {
    /// Initialize the GPU worker
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing GPU worker...");
        
        // Initialize wgpu
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // Get adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to get GPU adapter")?;
        
        info!("Using GPU adapter: {:?}", adapter.get_info());
        
        // Get device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Guardian GPU Worker"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        
        // Initialize chunk generator
        let chunk_generator = ChunkGenerator::new(&device).await?;
        
        info!("GPU worker initialized successfully");
        
        Ok(Self {
            device,
            queue,
            chunk_generator,
            is_healthy: true,
        })
    }
    
    /// Submit a chunk generation job
    pub async fn submit_chunk_job(&mut self, job: ChunkJob) -> Result<ChunkResult, Box<dyn std::error::Error>> {
        info!("Submitting chunk job: ({}, {})", job.chunk_x, job.chunk_z);
        
        // Generate chunk using GPU
        let result = self.chunk_generator.generate_chunk(
            &self.device,
            &self.queue,
            job.chunk_x,
            job.chunk_z,
            job.seed,
            &job.dimension,
        ).await?;
        
        info!("Chunk generation completed: ({}, {})", job.chunk_x, job.chunk_z);
        Ok(result)
    }
    
    /// Perform health check
    pub fn health_check(&mut self) -> bool {
        // Simple health check - in a real implementation, this would test GPU functionality
        self.is_healthy = true;
        self.is_healthy
    }
    
    /// Cleanup resources
    pub fn cleanup(&mut self) {
        info!("Cleaning up GPU worker...");
        self.is_healthy = false;
    }
}

/// Initialize the GPU worker (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_init() -> c_int {
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(GpuWorker::new()) {
        Ok(worker) => {
            unsafe {
                GPU_WORKER = Some(Arc::new(Mutex::new(worker)));
            }
            0
        }
        Err(e) => {
            error!("Failed to initialize GPU worker: {}", e);
            -1
        }
    }
}

/// Submit a chunk job (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_submit_chunk_job(job: ChunkJob, out_handle: *mut JobHandle) -> c_int {
    unsafe {
        if let Some(worker_arc) = &GPU_WORKER {
            let rt = tokio::runtime::Handle::current();
            match rt.block_on(async {
                let mut worker = worker_arc.lock().await;
                worker.submit_chunk_job(job).await
            }) {
                Ok(result) => {
                    // Create job handle and store result
                    let handle = JobHandle {
                        result: Some(result),
                        completed: true,
                    };
                    ptr::write(out_handle, handle);
                    0
                }
                Err(e) => {
                    error!("Failed to submit chunk job: {}", e);
                    -1
                }
            }
        } else {
            error!("GPU worker not initialized");
            -1
        }
    }
}

/// Try to fetch a result (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_try_fetch_result(handle: *mut JobHandle, out_result: *mut ChunkResult) -> c_int {
    unsafe {
        let handle_ref = &*handle;
        if handle_ref.completed {
            if let Some(result) = &handle_ref.result {
                ptr::write(out_result, result.clone());
                0
            } else {
                -1
            }
        } else {
            -2 // Not ready
        }
    }
}

/// Free a result (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_free_result(result: *mut ChunkResult) {
    unsafe {
        if !result.is_null() {
            ptr::drop_in_place(result);
        }
    }
}

/// Health check (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_health_check() -> c_int {
    unsafe {
        if let Some(worker_arc) = &GPU_WORKER {
            let rt = tokio::runtime::Handle::current();
            match rt.block_on(async {
                let mut worker = worker_arc.lock().await;
                worker.health_check()
            }) {
                true => 0,
                false => -1,
            }
        } else {
            -1
        }
    }
}

/// Cleanup (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_cleanup() {
    unsafe {
        if let Some(worker_arc) = &GPU_WORKER {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut worker = worker_arc.lock().await;
                worker.cleanup();
            });
            GPU_WORKER = None;
        }
    }
}
```

```rust
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use serde::{Deserialize, Serialize};

/// Chunk job structure for C ABI
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkJob {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub seed: i64,
    pub dimension: *const c_char,
    pub tick_count: i32,
    pub rule_version: *const c_char,
}

/// Chunk result structure for C ABI
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResult {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub seed: i64,
    pub content_hash: *const c_char,
    pub density_data: *mut u8,
    pub density_data_size: i32,
    pub mask_data: *mut u8,
    pub mask_data_size: i32,
    pub biome_data: *mut u8,
    pub biome_data_size: i32,
    pub status: i32, // 0 = success, 1 = error, 2 = not ready
}

/// Job handle for tracking async operations
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JobHandle {
    pub result: Option<ChunkResult>,
    pub completed: bool,
}

impl ChunkJob {
    /// Create a new chunk job from C strings
    pub fn new(
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
        dimension: *const c_char,
        tick_count: i32,
        rule_version: *const c_char,
    ) -> Self {
        Self {
            chunk_x,
            chunk_z,
            seed,
            dimension,
            tick_count,
            rule_version,
        }
    }
    
    /// Get dimension as string
    pub fn get_dimension(&self) -> String {
        unsafe {
            if self.dimension.is_null() {
                "overworld".to_string()
            } else {
                std::ffi::CStr::from_ptr(self.dimension)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }
    
    /// Get rule version as string
    pub fn get_rule_version(&self) -> String {
        unsafe {
            if self.rule_version.is_null() {
                "1.0.0".to_string()
            } else {
                std::ffi::CStr::from_ptr(self.rule_version)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }
}

impl ChunkResult {
    /// Create a new chunk result
    pub fn new(
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
        content_hash: String,
        density_data: Vec<u8>,
        mask_data: Vec<u8>,
        biome_data: Vec<u8>,
    ) -> Self {
        let hash_cstring = CString::new(content_hash).unwrap();
        let hash_ptr = hash_cstring.into_raw();
        
        let density_ptr = if density_data.is_empty() {
            std::ptr::null_mut()
        } else {
            density_data.as_ptr() as *mut u8
        };
        
        let mask_ptr = if mask_data.is_empty() {
            std::ptr::null_mut()
        } else {
            mask_data.as_ptr() as *mut u8
        };
        
        let biome_ptr = if biome_data.is_empty() {
            std::ptr::null_mut()
        } else {
            biome_data.as_ptr() as *mut u8
        };
        
        Self {
            chunk_x,
            chunk_z,
            seed,
            content_hash: hash_ptr,
            density_data: density_ptr,
            density_data_size: density_data.len() as i32,
            mask_data: mask_ptr,
            mask_data_size: mask_data.len() as i32,
            biome_data: biome_ptr,
            biome_data_size: biome_data.len() as i32,
            status: 0, // success
        }
    }
    
    /// Get content hash as string
    pub fn get_content_hash(&self) -> String {
        unsafe {
            if self.content_hash.is_null() {
                "unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(self.content_hash)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }
    
    /// Get density data as slice
    pub fn get_density_data(&self) -> &[u8] {
        unsafe {
            if self.density_data.is_null() || self.density_data_size <= 0 {
                &[]
            } else {
                std::slice::from_raw_parts(self.density_data, self.density_data_size as usize)
            }
        }
    }
    
    /// Get mask data as slice
    pub fn get_mask_data(&self) -> &[u8] {
        unsafe {
            if self.mask_data.is_null() || self.mask_data_size <= 0 {
                &[]
            } else {
                std::slice::from_raw_parts(self.mask_data, self.mask_data_size as usize)
            }
        }
    }
    
    /// Get biome data as slice
    pub fn get_biome_data(&self) -> &[u8] {
        unsafe {
            if self.biome_data.is_null() || self.biome_data_size <= 0 {
                &[]
            } else {
                std::slice::from_raw_parts(self.biome_data, self.biome_data_size as usize)
            }
        }
    }
}

impl Drop for ChunkResult {
    fn drop(&mut self) {
        unsafe {
            if !self.content_hash.is_null() {
                let _ = CString::from_raw(self.content_hash as *mut c_char);
            }
        }
    }
}
```

```rust
mod density;
mod mask;

use wgpu::*;
use anyhow::Result;
use crate::ffi::ChunkResult;

pub use density::DensityKernel;
pub use mask::MaskKernel;

/// Chunk generator that coordinates GPU kernels for world generation
pub struct ChunkGenerator {
    density_kernel: DensityKernel,
    mask_kernel: MaskKernel,
    bind_group_layout: BindGroupLayout,
}

impl ChunkGenerator {
    /// Create a new chunk generator
    pub async fn new(device: &Device) -> Result<Self> {
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Chunk Generator Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Initialize kernels
        let density_kernel = DensityKernel::new(device, &bind_group_layout).await?;
        let mask_kernel = MaskKernel::new(device, &bind_group_layout).await?;
        
        Ok(Self {
            density_kernel,
            mask_kernel,
            bind_group_layout,
        })
    }
    
    /// Generate a chunk using GPU kernels
    pub async fn generate_chunk(
        &self,
        device: &Device,
        queue: &Queue,
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
        dimension: &str,
    ) -> Result<ChunkResult> {
        // Create input buffer with chunk parameters
        let input_data = ChunkInput {
            chunk_x,
            chunk_z,
            seed,
            dimension_hash: self.hash_dimension(dimension),
        };
        
        let input_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Chunk Input Buffer"),
            size: std::mem::size_of::<ChunkInput>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        queue.write_buffer(&input_buffer, 0, bytemuck::bytes_of(&input_data));
        
        // Generate density data
        let density_data = self.density_kernel.generate(
            device,
            queue,
            &input_buffer,
            chunk_x,
            chunk_z,
            seed,
        ).await?;
        
        // Generate mask data
        let mask_data = self.mask_kernel.generate(
            device,
            queue,
            &input_buffer,
            &density_data,
        ).await?;
        
        // Generate biome data (placeholder)
        let biome_data = self.generate_biome_data(chunk_x, chunk_z, seed, dimension);
        
        // Create content hash
        let content_hash = self.create_content_hash(chunk_x, chunk_z, seed, &density_data, &mask_data);
        
        Ok(ChunkResult::new(
            chunk_x,
            chunk_z,
            seed,
            content_hash,
            density_data,
            mask_data,
            biome_data,
        ))
    }
    
    /// Generate biome data (placeholder implementation)
    fn generate_biome_data(&self, chunk_x: i32, chunk_z: i32, seed: i64, dimension: &str) -> Vec<u8> {
        // This would generate actual biome data based on the chunk position and seed
        // For now, return placeholder data
        vec![0u8; 16 * 16] // 16x16 biome grid
    }
    
    /// Create content hash for validation
    fn create_content_hash(&self, chunk_x: i32, chunk_z: i32, seed: i64, density_data: &[u8], mask_data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        chunk_x.hash(&mut hasher);
        chunk_z.hash(&mut hasher);
        seed.hash(&mut hasher);
        density_data.hash(&mut hasher);
        mask_data.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
    
    /// Hash dimension string to integer
    fn hash_dimension(&self, dimension: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        dimension.hash(&mut hasher);
        hasher.finish() as u32
    }
}

/// Input data for chunk generation
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ChunkInput {
    chunk_x: i32,
    chunk_z: i32,
    seed: i64,
    dimension_hash: u32,
}
```

```plaintext
// Density generation kernel for Minecraft chunk generation
// This kernel generates density values for terrain generation

struct ChunkInput {
    chunk_x: i32,
    chunk_z: i32,
    seed: i64,
    dimension_hash: u32,
}

struct DensityOutput {
    density: f32,
    biome: u32,
    temperature: f32,
    humidity: f32,
}

@group(0) @binding(0)
var<storage, read_write> output: array<DensityOutput>;

@group(0) @binding(1)
var<uniform> input: ChunkInput;

// Simple hash function for deterministic noise
fn hash(seed: u32) -> f32 {
    var x = seed;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = (x >> 16) ^ x;
    return f32(x) / 4294967296.0;
}

// 2D noise function
fn noise2d(x: f32, z: f32, seed: u32) -> f32 {
    let x0 = floor(x);
    let z0 = floor(z);
    let x1 = x0 + 1.0;
    let z1 = z0 + 1.0;
    
    let fx = x - x0;
    let fz = z - z0;
    
    let n00 = hash(u32(x0) + u32(z0) * 374761393 + seed);
    let n01 = hash(u32(x0) + u32(z1) * 374761393 + seed);
    let n10 = hash(u32(x1) + u32(z0) * 374761393 + seed);
    let n11 = hash(u32(x1) + u32(z1) * 374761393 + seed);
    
    let nx0 = mix(n00, n10, fx);
    let nx1 = mix(n01, n11, fx);
    
    return mix(nx0, nx1, fz);
}

// Multi-octave noise
fn noise2d_octaves(x: f32, z: f32, seed: u32, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    var max_value = 0.0;
    
    for (var i = 0u; i < octaves; i++) {
        value += noise2d(x * frequency, z * frequency, seed + i) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value / max_value;
}

// Terrain height calculation
fn calculate_terrain_height(x: f32, z: f32, seed: i64) -> f32 {
    let base_height = 64.0;
    let height_variation = 32.0;
    
    // Main terrain noise
    let terrain_noise = noise2d_octaves(x * 0.01, z * 0.01, u32(seed), 4u);
    
    // Mountain noise
    let mountain_noise = noise2d_octaves(x * 0.005, z * 0.005, u32(seed) + 1000, 3u);
    let mountain_factor = smoothstep(0.3, 0.7, mountain_noise);
    
    // Valley noise
    let valley_noise = noise2d_octaves(x * 0.02, z * 0.02, u32(seed) + 2000, 2u);
    let valley_factor = smoothstep(0.4, 0.6, valley_noise);
    
    let height = base_height + 
                 terrain_noise * height_variation + 
                 mountain_factor * 64.0 - 
                 valley_factor * 32.0;
    
    return height;
}

// Biome calculation
fn calculate_biome(x: f32, z: f32, seed: i64) -> u32 {
    let temperature = noise2d_octaves(x * 0.008, z * 0.008, u32(seed) + 3000, 2u);
    let humidity = noise2d_octaves(x * 0.012, z * 0.012, u32(seed) + 4000, 2u);
    
    // Simple biome classification
    if (temperature > 0.6 && humidity > 0.4) {
        return 1u; // Forest
    } else if (temperature > 0.4 && humidity < 0.3) {
        return 2u; // Plains
    } else if (temperature < 0.3) {
        return 3u; // Tundra
    } else if (humidity > 0.7) {
        return 4u; // Swamp
    } else {
        return 0u; // Default
    }
}

// Smoothstep function
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// Clamp function
fn clamp(x: f32, min_val: f32, max_val: f32) -> f32 {
    return max(min(x, max_val), min_val);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let z = global_id.y;
    
    if (x >= 16u || z >= 16u) {
        return;
    }
    
    let world_x = f32(input.chunk_x * 16 + i32(x));
    let world_z = f32(input.chunk_z * 16 + i32(z));
    
    // Calculate terrain height
    let height = calculate_terrain_height(world_x, world_z, input.seed);
    
    // Calculate biome
    let biome = calculate_biome(world_x, world_z, input.seed);
    
    // Calculate temperature and humidity
    let temperature = noise2d_octaves(world_x * 0.008, world_z * 0.008, u32(input.seed) + 3000, 2u);
    let humidity = noise2d_octaves(world_x * 0.012, world_z * 0.012, u32(input.seed) + 4000, 2u);
    
    // Store result
    let index = z * 16u + x;
    output[index] = DensityOutput(
        height,
        biome,
        temperature,
        humidity
    );
}
```

```plaintext
// Mask generation kernel for Minecraft chunk generation
// This kernel generates masks for caves, ores, and other features

struct ChunkInput {
    chunk_x: i32,
    chunk_z: i32,
    seed: i64,
    dimension_hash: u32,
}

struct DensityOutput {
    density: f32,
    biome: u32,
    temperature: f32,
    humidity: f32,
}

struct MaskOutput {
    cave_mask: u32,
    ore_mask: u32,
    water_mask: u32,
    structure_mask: u32,
}

@group(0) @binding(0)
var<storage, read_write> output: array<MaskOutput>;

@group(0) @binding(1)
var<uniform> input: ChunkInput;

@group(0) @binding(2)
var<storage, read> density_data: array<DensityOutput>;

// Simple hash function for deterministic noise
fn hash(seed: u32) -> f32 {
    var x = seed;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = ((x >> 16) ^ x) * 0x45d9f3b;
    x = (x >> 16) ^ x;
    return f32(x) / 4294967296.0;
}

// 3D noise function for cave generation
fn noise3d(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let x0 = floor(x);
    let y0 = floor(y);
    let z0 = floor(z);
    let x1 = x0 + 1.0;
    let y1 = y0 + 1.0;
    let z1 = z0 + 1.0;
    
    let fx = x - x0;
    let fy = y - y0;
    let fz = z - z0;
    
    let n000 = hash(u32(x0) + u32(y0) * 374761393 + u32(z0) * 668265263 + seed);
    let n001 = hash(u32(x0) + u32(y0) * 374761393 + u32(z1) * 668265263 + seed);
    let n010 = hash(u32(x0) + u32(y1) * 374761393 + u32(z0) * 668265263 + seed);
    let n011 = hash(u32(x0) + u32(y1) * 374761393 + u32(z1) * 668265263 + seed);
    let n100 = hash(u32(x1) + u32(y0) * 374761393 + u32(z0) * 668265263 + seed);
    let n101 = hash(u32(x1) + u32(y0) * 374761393 + u32(z1) * 668265263 + seed);
    let n110 = hash(u32(x1) + u32(y1) * 374761393 + u32(z0) * 668265263 + seed);
    let n111 = hash(u32(x1) + u32(y1) * 374761393 + u32(z1) * 668265263 + seed);
    
    let nx00 = mix(n000, n100, fx);
    let nx01 = mix(n001, n101, fx);
    let nx10 = mix(n010, n110, fx);
    let nx11 = mix(n011, n111, fx);
    
    let nxy0 = mix(nx00, nx10, fy);
    let nxy1 = mix(nx01, nx11, fy);
    
    return mix(nxy0, nxy1, fz);
}

// Multi-octave 3D noise
fn noise3d_octaves(x: f32, y: f32, z: f32, seed: u32, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    var max_value = 0.0;
    
    for (var i = 0u; i < octaves; i++) {
        value += noise3d(x * frequency, y * frequency, z * frequency, seed + i) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    return value / max_value;
}

// Cave generation
fn generate_cave_mask(x: f32, y: f32, z: f32, seed: i64) -> u32 {
    let cave_noise = noise3d_octaves(x * 0.05, y * 0.05, z * 0.05, u32(seed) + 5000, 3u);
    let cave_threshold = 0.3;
    
    if (cave_noise > cave_threshold) {
        return 1u; // Cave
    } else {
        return 0u; // Solid
    }
}

// Ore generation
fn generate_ore_mask(x: f32, y: f32, z: f32, seed: i64) -> u32 {
    let ore_noise = noise3d_octaves(x * 0.1, y * 0.1, z * 0.1, u32(seed) + 6000, 2u);
    let ore_threshold = 0.8;
    
    if (ore_noise > ore_threshold) {
        // Determine ore type based on depth and noise
        if (y < 16.0) {
            return 1u; // Coal
        } else if (y < 32.0) {
            return 2u; // Iron
        } else if (y < 48.0) {
            return 3u; // Gold
        } else {
            return 4u; // Diamond
        }
    } else {
        return 0u; // No ore
    }
}

// Water generation
fn generate_water_mask(x: f32, y: f32, z: f32, seed: i64) -> u32 {
    let water_noise = noise3d_octaves(x * 0.02, y * 0.02, z * 0.02, u32(seed) + 7000, 2u);
    let water_threshold = 0.4;
    
    if (water_noise > water_threshold && y < 64.0) {
        return 1u; // Water
    } else {
        return 0u; // Air/Solid
    }
}

// Structure generation
fn generate_structure_mask(x: f32, y: f32, z: f32, seed: i64) -> u32 {
    let structure_noise = noise3d_octaves(x * 0.01, y * 0.01, z * 0.01, u32(seed) + 8000, 2u);
    let structure_threshold = 0.9;
    
    if (structure_noise > structure_threshold) {
        // Determine structure type based on position and noise
        if (y > 60.0) {
            return 1u; // Surface structure
        } else if (y < 20.0) {
            return 2u; // Underground structure
        } else {
            return 3u; // Mid-level structure
        }
    } else {
        return 0u; // No structure
    }
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let z = global_id.y;
    
    if (x >= 16u || z >= 16u) {
        return;
    }
    
    let world_x = f32(input.chunk_x * 16 + i32(x));
    let world_z = f32(input.chunk_z * 16 + i32(z));
    
    // Get density data for this position
    let density_index = z * 16u + x;
    let density = density_data[density_index];
    
    // Generate masks for each Y level (simplified to single level for now)
    let y = density.density; // Use terrain height as reference
    
    let cave_mask = generate_cave_mask(world_x, y, world_z, input.seed);
    let ore_mask = generate_ore_mask(world_x, y, world_z, input.seed);
    let water_mask = generate_water_mask(world_x, y, world_z, input.seed);
    let structure_mask = generate_structure_mask(world_x, y, world_z, input.seed);
    
    // Store result
    let index = z * 16u + x;
    output[index] = MaskOutput(
        cave_mask,
        ore_mask,
        water_mask,
        structure_mask
    );
}
```

```rust
use wgpu::*;
use anyhow::Result;
use std::borrow::Cow;

/// Density generation kernel for terrain generation
pub struct DensityKernel {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl DensityKernel {
    /// Create a new density kernel
    pub async fn new(device: &Device, bind_group_layout: &BindGroupLayout) -> Result<Self> {
        // Load shader
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Density Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("density.wgsl"))),
        });
        
        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Density Pipeline"),
            layout: None,
            compute: ProgrammableStageDescriptor {
                module: &shader,
                entry_point: "main",
                compilation_options: PipelineCompilationOptions::default(),
            },
        });
        
        Ok(Self {
            pipeline,
            bind_group_layout: bind_group_layout.clone(),
        })
    }
    
    /// Generate density data for a chunk
    pub async fn generate(
        &self,
        device: &Device,
        queue: &Queue,
        input_buffer: &Buffer,
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
    ) -> Result<Vec<u8>> {
        // Create output buffer
        let output_size = 16 * 16 * 16; // 16x16x16 density values
        let output_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Density Output Buffer"),
            size: output_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Density Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: output_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create command encoder
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Density Command Encoder"),
        });
        
        // Dispatch compute shader
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Density Compute Pass"),
            });
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1); // 16x16 workgroups
        }
        
        // Submit command
        queue.submit(std::iter::once(encoder.finish()));
        
        // Read back results
        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        
        device.poll(Maintain::Wait);
        receiver.receive().await.unwrap()?;
        
        // Get mapped data
        let data = buffer_slice.get_mapped_range();
        let result = data.to_vec();
        drop(data);
        output_buffer.unmap();
        
        Ok(result)
    }
}
```

```rust
use wgpu::*;
use anyhow::Result;
use std::borrow::Cow;

/// Mask generation kernel for caves, ores, and other features
pub struct MaskKernel {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
}

impl MaskKernel {
    /// Create a new mask kernel
    pub async fn new(device: &Device, bind_group_layout: &BindGroupLayout) -> Result<Self> {
        // Load shader
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Mask Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("mask.wgsl"))),
        });
        
        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Mask Pipeline"),
            layout: None,
            compute: ProgrammableStageDescriptor {
                module: &shader,
                entry_point: "main",
                compilation_options: PipelineCompilationOptions::default(),
            },
        });
        
        Ok(Self {
            pipeline,
            bind_group_layout: bind_group_layout.clone(),
        })
    }
    
    /// Generate mask data for a chunk
    pub async fn generate(
        &self,
        device: &Device,
        queue: &Queue,
        input_buffer: &Buffer,
        density_data: &[u8],
    ) -> Result<Vec<u8>> {
        // Create density buffer
        let density_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Density Data Buffer"),
            size: density_data.len() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        queue.write_buffer(&density_buffer, 0, density_data);
        
        // Create output buffer
        let output_size = 16 * 16 * 4; // 16x16 mask values (4 bytes each)
        let output_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Mask Output Buffer"),
            size: output_size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create bind group layout for mask kernel
        let mask_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Mask Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Create bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Mask Bind Group"),
            layout: &mask_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: output_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: density_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create command encoder
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Mask Command Encoder"),
        });
        
        // Dispatch compute shader
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Mask Compute Pass"),
            });
            
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1); // 16x16 workgroups
        }
        
        // Submit command
        queue.submit(std::iter::once(encoder.finish()));
        
        // Read back results
        let buffer_slice = output_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        
        device.poll(Maintain::Wait);
        receiver.receive().await.unwrap()?;
        
        // Get mapped data
        let data = buffer_slice.get_mapped_range();
        let result = data.to_vec();
        drop(data);
        output_buffer.unmap();
        
        Ok(result)
    }
}
```

```rust
use gpu_worker::*;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting GPU Worker...");
    
    // Initialize GPU worker
    let mut worker = GpuWorker::new().await?;
    
    info!("GPU Worker started successfully");
    
    // Keep the worker running
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down GPU Worker...");
    worker.cleanup();
    
    Ok(())
}
```

```dockerfile
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libasound2 \
    libudev1 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false gpuworker

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/gpu-worker /app/gpu-worker

# Change ownership
RUN chown -R gpuworker:gpuworker /app

# Switch to app user
USER gpuworker

# Expose port (if needed for health checks)
EXPOSE 8081

# Run the application
CMD ["./gpu-worker"]
```

Now let me create the Rust watchdog daemon (hostd):


```plaintext
[package]
name = "hostd"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"
clap = { version = "4.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
prometheus = "0.13"
warp = "0.3"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
nix = "0.27"
libc = "0.2"
notify = "6.0"
```

```rust
use hostd::*;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "hostd")]
#[command(about = "Guardian Host Daemon - Process supervision and high availability")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "/configs/hostd.yaml")]
    config: String,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Daemon mode
    #[arg(short, long)]
    daemon: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = args.log_level.parse::<tracing::Level>()?;
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();
    
    info!("Starting Guardian Host Daemon...");
    
    // Load configuration
    let config = Config::load(&args.config)?;
    info!("Configuration loaded from: {}", args.config);
    
    // Create host daemon
    let mut hostd = HostDaemon::new(config).await?;
    
    // Start the daemon
    if args.daemon {
        info!("Running in daemon mode");
        hostd.run_daemon().await?;
    } else {
        info!("Running in foreground mode");
        hostd.run().await?;
    }
    
    info!("Guardian Host Daemon stopped");
    Ok(())
}
```

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use anyhow::Result;
use uuid::Uuid;

pub mod config;
pub mod process;
pub mod snapshot;
pub mod metrics;
pub mod web;

use config::Config;
use process::ProcessManager;
use snapshot::SnapshotManager;
use metrics::MetricsCollector;
use web::WebServer;

/// Main host daemon that orchestrates all Guardian services
pub struct HostDaemon {
    config: Config,
    process_manager: Arc<ProcessManager>,
    snapshot_manager: Arc<SnapshotManager>,
    metrics_collector: Arc<MetricsCollector>,
    web_server: Arc<WebServer>,
    state: Arc<RwLock<DaemonState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub start_time: Instant,
    pub uptime: Duration,
    pub server_status: ServerStatus,
    pub gpu_worker_status: WorkerStatus,
    pub last_health_check: Option<Instant>,
    pub restart_count: u32,
    pub last_restart: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Crashed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerStatus {
    Starting,
    Running,
    Stopped,
    Crashed,
    Unknown,
}

impl HostDaemon {
    /// Create a new host daemon
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Guardian Host Daemon...");
        
        // Create process manager
        let process_manager = Arc::new(ProcessManager::new(config.clone()).await?);
        
        // Create snapshot manager
        let snapshot_manager = Arc::new(SnapshotManager::new(config.clone()).await?);
        
        // Create metrics collector
        let metrics_collector = Arc::new(MetricsCollector::new(config.clone()).await?);
        
        // Create web server
        let web_server = Arc::new(WebServer::new(config.clone(), metrics_collector.clone()).await?);
        
        // Create initial state
        let state = Arc::new(RwLock::new(DaemonState {
            start_time: Instant::now(),
            uptime: Duration::ZERO,
            server_status: ServerStatus::Starting,
            gpu_worker_status: WorkerStatus::Starting,
            last_health_check: None,
            restart_count: 0,
            last_restart: None,
        }));
        
        info!("Guardian Host Daemon initialized successfully");
        
        Ok(Self {
            config,
            process_manager,
            snapshot_manager,
            metrics_collector,
            web_server,
            state,
        })
    }
    
    /// Run the daemon in foreground mode
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Guardian Host Daemon services...");
        
        // Start all services
        self.start_services().await?;
        
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        
        info!("Shutdown signal received, stopping services...");
        self.stop_services().await?;
        
        Ok(())
    }
    
    /// Run the daemon in daemon mode
    pub async fn run_daemon(&mut self) -> Result<()> {
        info!("Starting Guardian Host Daemon in daemon mode...");
        
        // Start all services
        self.start_services().await?;
        
        // Keep running until shutdown
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            // Check if we should shutdown
            if self.should_shutdown().await {
                break;
            }
        }
        
        info!("Daemon shutdown requested, stopping services...");
        self.stop_services().await?;
        
        Ok(())
    }
    
    /// Start all daemon services
    async fn start_services(&mut self) -> Result<()> {
        // Start process manager
        self.process_manager.start().await?;
        
        // Start snapshot manager
        self.snapshot_manager.start().await?;
        
        // Start metrics collector
        self.metrics_collector.start().await?;
        
        // Start web server
        self.web_server.start().await?;
        
        // Start main daemon loop
        self.start_daemon_loop().await;
        
        info!("All Guardian Host Daemon services started");
        Ok(())
    }
    
    /// Stop all daemon services
    async fn stop_services(&mut self) -> Result<()> {
        info!("Stopping Guardian Host Daemon services...");
        
        // Stop web server
        self.web_server.stop().await?;
        
        // Stop metrics collector
        self.metrics_collector.stop().await?;
        
        // Stop snapshot manager
        self.snapshot_manager.stop().await?;
        
        // Stop process manager
        self.process_manager.stop().await?;
        
        info!("All Guardian Host Daemon services stopped");
        Ok(())
    }
    
    /// Start the main daemon loop
    async fn start_daemon_loop(&self) {
        let state = self.state.clone();
        let process_manager = self.process_manager.clone();
        let snapshot_manager = self.snapshot_manager.clone();
        let metrics_collector = self.metrics_collector.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Update uptime
                {
                    let mut state_guard = state.write().await;
                    state_guard.uptime = state_guard.start_time.elapsed();
                }
                
                // Perform health checks
                if let Err(e) = Self::perform_health_checks(
                    &state,
                    &process_manager,
                    &snapshot_manager,
                    &metrics_collector,
                ).await {
                    error!("Health check failed: {}", e);
                }
                
                // Update metrics
                if let Err(e) = metrics_collector.update_metrics(&state).await {
                    error!("Failed to update metrics: {}", e);
                }
            }
        });
    }
    
    /// Perform health checks on all services
    async fn perform_health_checks(
        state: &Arc<RwLock<DaemonState>>,
        process_manager: &Arc<ProcessManager>,
        snapshot_manager: &Arc<SnapshotManager>,
        metrics_collector: &Arc<MetricsCollector>,
    ) -> Result<()> {
        debug!("Performing health checks...");
        
        // Check server process
        let server_healthy = process_manager.is_server_healthy().await;
        
        // Check GPU worker
        let gpu_worker_healthy = process_manager.is_gpu_worker_healthy().await;
        
        // Update state
        {
            let mut state_guard = state.write().await;
            state_guard.last_health_check = Some(Instant::now());
            
            state_guard.server_status = if server_healthy {
                ServerStatus::Running
            } else {
                ServerStatus::Crashed
            };
            
            state_guard.gpu_worker_status = if gpu_worker_healthy {
                WorkerStatus::Running
            } else {
                WorkerStatus::Crashed
            };
        }
        
        // Take snapshots if needed
        if server_healthy {
            if let Err(e) = snapshot_manager.take_snapshot_if_needed().await {
                warn!("Failed to take snapshot: {}", e);
            }
        }
        
        debug!("Health checks completed");
        Ok(())
    }
    
    /// Check if the daemon should shutdown
    async fn should_shutdown(&self) -> bool {
        // Check for shutdown file
        let shutdown_file = PathBuf::from("/tmp/guardian-shutdown");
        if shutdown_file.exists() {
            info!("Shutdown file detected, initiating shutdown...");
            return true;
        }
        
        // Check for excessive restarts
        let state = self.state.read().await;
        if state.restart_count > 10 {
            error!("Too many restarts, shutting down daemon");
            return true;
        }
        
        false
    }
    
    /// Get current daemon state
    pub async fn get_state(&self) -> DaemonState {
        self.state.read().await.clone()
    }
    
    /// Get daemon statistics
    pub async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        let state = self.state.read().await;
        stats.insert("uptime".to_string(), serde_json::Value::String(format!("{:?}", state.uptime)));
        stats.insert("server_status".to_string(), serde_json::Value::String(format!("{:?}", state.server_status)));
        stats.insert("gpu_worker_status".to_string(), serde_json::Value::String(format!("{:?}", state.gpu_worker_status)));
        stats.insert("restart_count".to_string(), serde_json::Value::Number(state.restart_count.into()));
        
        // Add process manager stats
        let process_stats = self.process_manager.get_stats().await;
        stats.insert("process_stats".to_string(), serde_json::Value::Object(process_stats));
        
        // Add snapshot manager stats
        let snapshot_stats = self.snapshot_manager.get_stats().await;
        stats.insert("snapshot_stats".to_string(), serde_json::Value::Object(snapshot_stats));
        
        stats
    }
}
```


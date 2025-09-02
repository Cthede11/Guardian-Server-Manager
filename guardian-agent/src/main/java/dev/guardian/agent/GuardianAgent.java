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

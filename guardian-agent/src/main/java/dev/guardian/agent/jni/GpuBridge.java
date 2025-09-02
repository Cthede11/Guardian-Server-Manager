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

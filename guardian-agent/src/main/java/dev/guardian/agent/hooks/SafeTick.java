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

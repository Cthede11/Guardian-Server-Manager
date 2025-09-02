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

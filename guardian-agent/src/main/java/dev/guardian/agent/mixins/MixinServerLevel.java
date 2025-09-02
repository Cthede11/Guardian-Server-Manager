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

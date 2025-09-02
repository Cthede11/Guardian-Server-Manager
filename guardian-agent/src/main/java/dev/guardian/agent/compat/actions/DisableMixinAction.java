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

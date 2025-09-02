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

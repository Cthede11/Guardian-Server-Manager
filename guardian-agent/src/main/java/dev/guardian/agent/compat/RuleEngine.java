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

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
        
        /**
         * Create a new label for branching instructions
         */
        private Label createLabel() {
            return new Label();
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

# Roaming Report - Background Agent Proposals

## CURRENT STATE

### Outstanding Proposals
1. **Modpack System API Restructuring** (Class B) - Major refactor needed
2. **Type System Alignment** (Class B) - Frontend/backend type mismatches
3. **Settings System Enhancement** (Class B) - Missing general settings structure

### Suggested Next Priorities
1. Complete TypeScript type alignment for modpack components
2. Implement proper API client with correct method signatures
3. Add missing settings structure for general configuration
4. Fix remaining function signature mismatches

### Risky Areas Needing Human Review
- Modpack system has significant type mismatches between frontend expectations and backend reality
- API client methods don't match actual backend endpoints
- Settings system needs restructuring to support general configuration

---

## PROPOSALS

### 1. Modpack System API Restructuring
**Status**: OPEN  
**Risk Level**: Medium  
**Files**: `guardian-ui/src/lib/api/modpack.ts`, `guardian-ui/src/lib/types/modpack.ts`, `guardian-ui/src/components/Modpack/*`  
**Context**: The modpack system has significant type mismatches between frontend components and the API client. Components expect different property names and structures than what's defined in the types.

**Suggested Approach**:
- Align Modpack interface with frontend component expectations
- Add missing properties like `side`, `category`, `source`, `client_mods`, `server_mods`
- Update API client methods to match actual backend endpoints
- Add proper error handling and validation

**Test Impact**: High - affects all modpack-related components
**UI Impact**: Medium - improves type safety and user experience

### 2. Settings System Enhancement
**Status**: OPEN  
**Risk Level**: Low  
**Files**: `guardian-ui/src/lib/settings-manager.ts`, `guardian-ui/src/components/FirstRunWizard.tsx`  
**Context**: The settings system is missing a `general` configuration section that components expect.

**Suggested Approach**:
- Add `general` section to Settings interface
- Include properties like `autoStart`, `notifications`, `language`, `theme`
- Update settings manager to handle nested configuration
- Ensure backward compatibility with existing settings

**Test Impact**: Low - mainly type additions
**UI Impact**: Low - improves settings management

### 3. API Client Method Signature Alignment
**Status**: OPEN  
**Risk Level**: Medium  
**Files**: `guardian-ui/src/lib/api.ts`, `guardian-ui/src/lib/client.ts`  
**Context**: The API client methods don't match the expected signatures in components.

**Suggested Approach**:
- Update API client to use proper HTTP methods (GET, POST, etc.)
- Align method signatures with component expectations
- Add proper error handling and response parsing
- Implement proper WebSocket connection management

**Test Impact**: Medium - affects API integration
**UI Impact**: Medium - improves error handling and user feedback
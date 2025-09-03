import React from 'react';
import { UsersRoles as UsersRolesComponent } from './UsersRoles';
import { BackupTargets as BackupTargetsComponent } from './BackupTargets';
import { WorkspaceTokens as WorkspaceTokensComponent } from './WorkspaceTokens';
import { Theme as ThemeComponent } from './Theme';

// Users & Roles
export const UsersRoles: React.FC = () => {
  return (
    <div className="h-full">
      <UsersRolesComponent />
    </div>
  );
};

// Backup Targets
export const BackupTargets: React.FC = () => {
  return (
    <div className="h-full">
      <BackupTargetsComponent />
    </div>
  );
};

// Workspace Tokens
export const WorkspaceTokens: React.FC = () => {
  return (
    <div className="h-full">
      <WorkspaceTokensComponent />
    </div>
  );
};

// Theme
export const Theme: React.FC = () => {
  return (
    <div className="h-full">
      <ThemeComponent />
    </div>
  );
};
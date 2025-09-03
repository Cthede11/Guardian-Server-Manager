import React from 'react';
import { GeneralSettings } from '@/components/Settings/GeneralSettings';
import { JVMSettings } from '@/components/Settings/JVMSettings';
import { GPUSettings } from '@/components/Settings/GPUSettings';
import { HASettings } from '@/components/Settings/HASettings';
import { PathsSettings } from '@/components/Settings/PathsSettings';
import { ComposerSettings } from '@/components/Settings/ComposerSettings';
import { TokensSettings } from '@/components/Settings/TokensSettings';

// General settings
export const General: React.FC = () => {
  return (
    <div className="h-full">
      <GeneralSettings />
    </div>
  );
};

// JVM settings
export const JVM: React.FC = () => {
  return (
    <div className="h-full">
      <JVMSettings />
    </div>
  );
};

// GPU settings
export const GPU: React.FC = () => {
  return (
    <div className="h-full">
      <GPUSettings />
    </div>
  );
};

// HA settings
export const HA: React.FC = () => {
  return (
    <div className="h-full">
      <HASettings />
    </div>
  );
};

// Paths settings
export const Paths: React.FC = () => {
  return (
    <div className="h-full">
      <PathsSettings />
    </div>
  );
};

// Composer settings
export const Composer: React.FC = () => {
  return (
    <div className="h-full">
      <ComposerSettings />
    </div>
  );
};

// Tokens settings
export const Tokens: React.FC = () => {
  return (
    <div className="h-full">
      <TokensSettings />
    </div>
  );
};
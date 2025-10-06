// Server Creation Wizard Component
import React, { useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

interface ServerCreationWizardProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onServerCreated?: (server: any) => void;
  onClose?: () => void;
}

export function ServerCreationWizard({ open, onOpenChange, onServerCreated, onClose }: ServerCreationWizardProps) {
  const [serverName, setServerName] = useState('');
  const [serverVersion, setServerVersion] = useState('');
  const [serverType, setServerType] = useState('vanilla');

  const handleCreate = async () => {
    try {
      // TODO: Implement actual server creation
      const newServer = {
        id: Date.now().toString(),
        name: serverName,
        version: serverVersion,
        type: serverType,
        status: 'offline'
      };
      
      onServerCreated?.(newServer);
      onOpenChange(false);
      onClose?.();
      
      // Reset form
      setServerName('');
      setServerVersion('');
      setServerType('vanilla');
    } catch (error) {
      console.error('Failed to create server:', error);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Create New Server</DialogTitle>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="name" className="text-right">
              Name
            </Label>
            <Input
              id="name"
              value={serverName}
              onChange={(e) => setServerName(e.target.value)}
              className="col-span-3"
              placeholder="My Server"
            />
          </div>
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="version" className="text-right">
              Version
            </Label>
            <Select value={serverVersion} onValueChange={setServerVersion}>
              <SelectTrigger className="col-span-3">
                <SelectValue placeholder="Select version" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1.21.1">1.21.1</SelectItem>
                <SelectItem value="1.21">1.21</SelectItem>
                <SelectItem value="1.20.6">1.20.6</SelectItem>
                <SelectItem value="1.20.1">1.20.1</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="type" className="text-right">
              Type
            </Label>
            <Select value={serverType} onValueChange={setServerType}>
              <SelectTrigger className="col-span-3">
                <SelectValue placeholder="Select type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="vanilla">Vanilla</SelectItem>
                <SelectItem value="forge">Forge</SelectItem>
                <SelectItem value="fabric">Fabric</SelectItem>
                <SelectItem value="quilt">Quilt</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={!serverName || !serverVersion}>
            Create Server
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
export type ConductorStatus = 'Online' | 'Offline' | 'Degraded' | 'Upgrading';

export interface ConductorInfo {
  id: string;
  name: string;
  status: ConductorStatus;
  version: string;
  uptime: number;
}

export interface AgentInfo {
  id: string;
  name: string;
  conductorId: string;
  status: 'Running' | 'Stopped' | 'Error';
  pid?: number;
}

export interface DesignSpec {
  id: string;
  name: string;
  type: string;
  requirements: SpecRequirement[];
  constraints: SpecConstraint[];
  interfaces: SpecInterface[];
}

export interface SpecRequirement {
  id: string;
  description: string;
  priority: 'Must' | 'Should' | 'May';
  status: 'Draft' | 'Approved' | 'Implemented' | 'Verified';
}

export interface SpecConstraint {
  id: string;
  description: string;
  category: string;
}

export interface SpecInterface {
  id: string;
  name: string;
  direction: 'Input' | 'Output' | 'InOut';
  width: number;
  type: string;
}

export interface LogEntry {
  timestamp: string;
  level: 'Trace' | 'Debug' | 'Info' | 'Warn' | 'Error';
  source: string;
  message: string;
  traceId?: string;
}

export interface TaskInfo {
  id: string;
  name: string;
  conductorId: string;
  progress: number;
  status: 'Pending' | 'Running' | 'Completed' | 'Failed';
  startTime?: string;
  endTime?: string;
}

export interface ConfigValue {
  key: string;
  value: string | number | boolean;
  type: 'string' | 'number' | 'boolean';
  description?: string;
  default?: string | number | boolean;
}

export interface WaveformSignal {
  id: string;
  fullName: string;
  displayName: string;
  bitWidth: number;
  signalType: 'Wire' | 'Reg' | 'Integer' | 'Real';
  scope: string;
}
import React from 'react';
import type { ConductorInfo, ConductorStatus } from '../types';

const statusColors: Record<ConductorStatus, string> = {
  Online: '#2d8f5e',
  Offline: '#737373',
  Degraded: '#e8a62c',
  Upgrading: '#2c7de8',
};

interface ConductorStatusCardProps {
  conductor: ConductorInfo;
  onSelect?: (id: string) => void;
  onAction?: (id: string, action: string) => void;
}

export const ConductorStatusCard: React.FC<ConductorStatusCardProps> = ({
  conductor,
  onSelect,
  onAction,
}) => {
  const handleClick = () => {
    onSelect?.(conductor.id);
  };

  return (
    <div
      className="hestia-conductor-card"
      onClick={handleClick}
      style={{
        border: `2px solid ${statusColors[conductor.status]}`,
        borderRadius: '8px',
        padding: '12px',
        margin: '4px 0',
        cursor: 'pointer',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <strong>{conductor.name}</strong>
        <span
          style={{
            backgroundColor: statusColors[conductor.status],
            color: '#fff',
            padding: '2px 8px',
            borderRadius: '12px',
            fontSize: '12px',
          }}
        >
          {conductor.status}
        </span>
      </div>
      <div style={{ fontSize: '12px', color: '#737373', marginTop: '4px' }}>
        v{conductor.version} • uptime {conductor.uptime}s
      </div>
      <div style={{ marginTop: '8px', display: 'flex', gap: '4px' }}>
        <button onClick={() => onAction?.(conductor.id, 'start')}>Start</button>
        <button onClick={() => onAction?.(conductor.id, 'stop')}>Stop</button>
        <button onClick={() => onAction?.(conductor.id, 'restart')}>Restart</button>
      </div>
    </div>
  );
};
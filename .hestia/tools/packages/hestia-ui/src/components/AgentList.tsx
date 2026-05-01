import React from 'react';
import type { AgentInfo } from '../types';

interface AgentListProps {
  agents: AgentInfo[];
  onStart?: (id: string) => void;
  onStop?: (id: string) => void;
}

export const AgentList: React.FC<AgentListProps> = ({ agents, onStart, onStop }) => {
  return (
    <div className="hestia-agent-list">
      {agents.map((agent) => (
        <div
          key={agent.id}
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            padding: '8px',
            borderBottom: '1px solid #e5e5e5',
          }}
        >
          <div>
            <span style={{ fontWeight: 500 }}>{agent.name}</span>
            <span style={{ fontSize: '12px', color: '#737373', marginLeft: '8px' }}>
              [{agent.conductorId}]
            </span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span
              style={{
                fontSize: '12px',
                color: agent.status === 'Running' ? '#2d8f5e' : agent.status === 'Error' ? '#e84d2c' : '#737373',
              }}
            >
              {agent.status}
              {agent.pid ? ` (PID: ${agent.pid})` : ''}
            </span>
            {agent.status === 'Running' ? (
              <button onClick={() => onStop?.(agent.id)}>Stop</button>
            ) : (
              <button onClick={() => onStart?.(agent.id)}>Start</button>
            )}
          </div>
        </div>
      ))}
    </div>
  );
};
import React, { useState } from 'react';
import type { DesignSpec, SpecRequirement } from '../types';

interface SpecViewerProps {
  spec: DesignSpec;
  onChange?: (spec: DesignSpec) => void;
}

export const SpecViewer: React.FC<SpecViewerProps> = ({ spec, onChange }) => {
  const [activeTab, setActiveTab] = useState<'requirements' | 'constraints' | 'interfaces'>('requirements');

  return (
    <div className="hestia-spec-viewer" style={{ fontFamily: 'monospace', fontSize: '14px' }}>
      <h3 style={{ color: '#e84d2c' }}>{spec.name}</h3>
      <div style={{ fontSize: '12px', color: '#737373' }}>Type: {spec.type}</div>

      <div style={{ display: 'flex', gap: '8px', margin: '12px 0' }}>
        <button
          onClick={() => setActiveTab('requirements')}
          style={{ fontWeight: activeTab === 'requirements' ? 'bold' : 'normal' }}
        >
          Requirements ({spec.requirements.length})
        </button>
        <button
          onClick={() => setActiveTab('constraints')}
          style={{ fontWeight: activeTab === 'constraints' ? 'bold' : 'normal' }}
        >
          Constraints ({spec.constraints.length})
        </button>
        <button
          onClick={() => setActiveTab('interfaces')}
          style={{ fontWeight: activeTab === 'interfaces' ? 'bold' : 'normal' }}
        >
          Interfaces ({spec.interfaces.length})
        </button>
      </div>

      {activeTab === 'requirements' && (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '2px solid #e5e5e5' }}>
              <th style={{ textAlign: 'left' }}>ID</th>
              <th style={{ textAlign: 'left' }}>Description</th>
              <th>Priority</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {spec.requirements.map((req) => (
              <tr key={req.id} style={{ borderBottom: '1px solid #e5e5e5' }}>
                <td>{req.id}</td>
                <td>{req.description}</td>
                <td style={{ textAlign: 'center' }}>{req.priority}</td>
                <td style={{ textAlign: 'center' }}>{req.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {activeTab === 'constraints' && (
        <ul>
          {spec.constraints.map((c) => (
            <li key={c.id}>
              <strong>{c.id}</strong> [{c.category}]: {c.description}
            </li>
          ))}
        </ul>
      )}

      {activeTab === 'interfaces' && (
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ borderBottom: '2px solid #e5e5e5' }}>
              <th style={{ textAlign: 'left' }}>Name</th>
              <th>Direction</th>
              <th>Width</th>
              <th>Type</th>
            </tr>
          </thead>
          <tbody>
            {spec.interfaces.map((iface) => (
              <tr key={iface.id} style={{ borderBottom: '1px solid #e5e5e5' }}>
                <td>{iface.name}</td>
                <td style={{ textAlign: 'center' }}>{iface.direction}</td>
                <td style={{ textAlign: 'center' }}>{iface.width}</td>
                <td style={{ textAlign: 'center' }}>{iface.type}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
};
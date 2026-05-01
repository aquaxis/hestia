import React, { useState } from 'react';
import type { ConfigValue } from '../types';

interface ConfigPanelProps {
  values: ConfigValue[];
  onChange?: (key: string, value: string | number | boolean) => void;
  onSave?: (values: ConfigValue[]) => void;
}

export const ConfigPanel: React.FC<ConfigPanelProps> = ({ values, onChange, onSave }) => {
  const [localValues, setLocalValues] = useState<Record<string, string | number | boolean>>(
    Object.fromEntries(values.map((v) => [v.key, v.value]))
  );

  const handleChange = (key: string, value: string | number | boolean) => {
    setLocalValues((prev) => ({ ...prev, [key]: value }));
    onChange?.(key, value);
  };

  return (
    <div className="hestia-config-panel" style={{ fontSize: '14px' }}>
      {values.map((config) => (
        <div
          key={config.key}
          style={{ display: 'flex', alignItems: 'center', padding: '8px 0', borderBottom: '1px solid #e5e5e5' }}
        >
          <div style={{ width: '200px' }}>
            <strong>{config.key}</strong>
            {config.description && (
              <div style={{ fontSize: '11px', color: '#737373' }}>{config.description}</div>
            )}
          </div>
          <div style={{ flex: 1 }}>
            {config.type === 'boolean' ? (
              <input
                type="checkbox"
                checked={localValues[config.key] as boolean}
                onChange={(e) => handleChange(config.key, e.target.checked)}
              />
            ) : config.type === 'number' ? (
              <input
                type="number"
                value={localValues[config.key] as number}
                onChange={(e) => handleChange(config.key, Number(e.target.value))}
                style={{ width: '100%', padding: '4px', borderRadius: '4px', border: '1px solid #d4d4d4' }}
              />
            ) : (
              <input
                type="text"
                value={localValues[config.key] as string}
                onChange={(e) => handleChange(config.key, e.target.value)}
                style={{ width: '100%', padding: '4px', borderRadius: '4px', border: '1px solid #d4d4d4' }}
              />
            )}
          </div>
          {config.default !== undefined && (
            <button
              style={{ marginLeft: '8px', fontSize: '11px' }}
              onClick={() => handleChange(config.key, config.default!)}
            >
              Reset
            </button>
          )}
        </div>
      ))}
      <div style={{ marginTop: '12px' }}>
        <button
          onClick={() => onSave?.(values)}
          style={{
            backgroundColor: '#e84d2c',
            color: '#fff',
            border: 'none',
            padding: '8px 16px',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Save Configuration
        </button>
      </div>
    </div>
  );
};
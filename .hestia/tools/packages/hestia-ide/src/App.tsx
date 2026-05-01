import React, { useState } from 'react';
import { ConductorStatusCard } from 'hestia-ui';
import type { ConductorInfo } from 'hestia-ui';

const CONDUCTORS: ConductorInfo[] = [
  { id: 'ai', name: 'ai-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
  { id: 'rtl', name: 'rtl-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
  { id: 'fpga', name: 'fpga-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
  { id: 'asic', name: 'asic-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
  { id: 'pcb', name: 'pcb-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
  { id: 'hal', name: 'hal-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
  { id: 'apps', name: 'apps-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
  { id: 'debug', name: 'debug-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
  { id: 'rag', name: 'rag-conductor', status: 'Offline', version: '0.1.0', uptime: 0 },
];

const App: React.FC = () => {
  const [conductors] = useState(CONDUCTORS);

  return (
    <div style={{ display: 'flex', height: '100vh' }}>
      {/* Sidebar */}
      <div style={{ width: '280px', backgroundColor: '#262626', borderRight: '1px solid #404040', overflowY: 'auto' }}>
        <h2 style={{ color: '#e84d2c', padding: '12px', borderBottom: '1px solid #404040' }}>
          Hestia IDE
        </h2>
        {conductors.map((c) => (
          <ConductorStatusCard key={c.id} conductor={c} />
        ))}
      </div>
      {/* Main content */}
      <div style={{ flex: 1, padding: '20px' }}>
        <h1 style={{ color: '#e84d2c', marginBottom: '16px' }}>Hestia IDE</h1>
        <p style={{ color: '#737373' }}>Hardware Engineering Stack for Tool Integration and Automation</p>
        <p style={{ color: '#737373', marginTop: '8px' }}>Select a conductor from the sidebar to get started.</p>
      </div>
    </div>
  );
};

export default App;
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import './App.css';
import { getRecognizedService } from './recognizedServices';
import type { Service } from './serviceSchema';
import { ServicesSchema } from './serviceSchema';
import type { SortDirection, SortField } from './sorting';
import { sortDirections, sortFields, sortServices } from './sorting';

function App() {
  const [services, setServices] = useState<Service[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [sortField, setSortField] = useState<SortField>('port');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');

  // Fetch services (used for initial and polling)
  const fetchServices = () => {
    fetch('http://localhost:3000/')
      .then((res) => res.json())
      .then((data) => {
        const result = ServicesSchema.safeParse(data);
        if (result.success) {
          setServices((prev) => {
            // Only update if data is different
            if (JSON.stringify(prev) !== JSON.stringify(result.data)) {
              return result.data;
            }
            return prev;
          });
          setLoading(false);
        } else {
          setError('Invalid data from server');
          setLoading(false);
        }
      })
      .catch(() => {
        setError('Failed to load services')
        setLoading(false)
      })
  };

  useEffect(() => {
    fetchServices();
    const interval = setInterval(fetchServices, 30000); // 30 seconds
    return () => clearInterval(interval);
  }, []);

  const sortedServices = sortServices(services, sortField, sortDirection);

  return (
    <main className="container">
      <h1>Local Services Dashboard</h1>
      <div style={{ display: 'flex', gap: 16, alignItems: 'center', marginBottom: 16 }}>
        <label>
          Sort by:{' '}
          <select value={sortField} onChange={e => setSortField(e.target.value as SortField)}>
            {sortFields.map(f => (
              <option key={f.value} value={f.value}>{f.label}</option>
            ))}
          </select>
        </label>
        <label>
          Order:{' '}
          <select value={sortDirection} onChange={e => setSortDirection(e.target.value as SortDirection)}>
            {sortDirections.map(d => (
              <option key={d.value} value={d.value}>{d.label}</option>
            ))}
          </select>
        </label>
      </div>
      {loading && <p>Loading...</p>}
      {error && <p style={{ color: 'red' }}>{error}</p>}
      {!loading && !error && (
        <table className="services-table">
          <thead>
            <tr>
              <th>Port</th>
              <th>Status</th>
              <th>Process</th>
              <th>PID</th>
              <th>User</th>
              <th>Command</th>
              <th>Type</th>
            </tr>
          </thead>
          <tbody>
            {sortedServices.map((svc) => {
              const recognized = getRecognizedService(svc);
              return (
                <tr key={svc.port}>
                  <td><Link to={`/${svc.port}`}>{svc.port}</Link></td>
                  <td>{svc.status}</td>
                  <td>{svc.process}</td>
                  <td>{svc.pid}</td>
                  <td>{svc.user}</td>
                  <td>{svc.command_line}</td>
                  <td>
                    {recognized && (
                      <span
                        style={{
                          background: recognized.color,
                          color: '#fff',
                          borderRadius: 4,
                          padding: '2px 8px',
                          fontSize: 12,
                          fontWeight: 600,
                        }}
                        title={recognized.label}
                      >
                        {recognized.label}
                      </span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
    </main>
  )
}

export default App;

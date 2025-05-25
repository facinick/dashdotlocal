import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import './App.css';

import { getRecognizedService } from '../../lib/recognizedServices';
import { type Service, ServicesSchema } from '../../lib/serviceSchema';
import type { SortDirection, SortField } from '../../lib/sorting';
import { sortDirections, sortFields } from '../../lib/sorting';

const PAGE_SIZE = 20;

function App() {
  const [services, setServices] = useState<Service[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [sortField, setSortField] = useState<SortField>('port');
  const [sortDirection, setSortDirection] = useState<SortDirection>('asc');
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const [pageSize, setPageSize] = useState(PAGE_SIZE);

  // Fetch services (used for initial and polling)
  const fetchServices = () => {
    setLoading(true);
    setError(null);
    const params = new URLSearchParams({
      sort_by: sortField,
      sort_order: sortDirection,
      page: String(page),
      page_size: String(pageSize),
    });
    fetch(`http://localhost:3000/?${params.toString()}`)
      .then((res) => res.json())
      .then((data) => {
        // Validate data shape
        if (!data || !Array.isArray(data.data) || typeof data.total !== 'number') {
          setError('Invalid data from server');
          setLoading(false);
          return;
        }
        const result = ServicesSchema.safeParse(data.data);
        if (result.success) {
          setServices(result.data);
          setTotal(data.total);
          setPage(data.page || 1);
          setPageSize(data.page_size || PAGE_SIZE);
          setLoading(false);
        } else {
          setError('Invalid data from server');
          setLoading(false);
        }
      })
      .catch(() => {
        setError('Failed to load services');
        setLoading(false);
      });
  };

  useEffect(() => {
    fetchServices();
    const interval = setInterval(fetchServices, 30000); // 30 seconds
    return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sortField, sortDirection, page, pageSize]);

  const totalPages = Math.ceil(total / pageSize);

  return (
    <main className="container">
      <h1>Local Services Dashboard</h1>
      <div style={{ display: 'flex', gap: 16, alignItems: 'center', marginBottom: 16 }}>
        <label>
          Sort by:{' '}
          <select value={sortField} onChange={e => { setSortField(e.target.value as SortField); setPage(1); }}>
            {sortFields.map(f => (
              <option key={f.value} value={f.value}>{f.label}</option>
            ))}
          </select>
        </label>
        <label>
          Order:{' '}
          <select value={sortDirection} onChange={e => { setSortDirection(e.target.value as SortDirection); setPage(1); }}>
            {sortDirections.map(d => (
              <option key={d.value} value={d.value}>{d.label}</option>
            ))}
          </select>
        </label>
        <label>
          Page size:{' '}
          <select value={pageSize} onChange={e => { setPageSize(Number(e.target.value)); setPage(1); }}>
            {[10, 20, 50, 100].map(size => (
              <option key={size} value={size}>{size}</option>
            ))}
          </select>
        </label>
      </div>
      <div style={{ marginBottom: 12 }}>
        <span>Total: {total}</span>
        {totalPages > 1 && (
          <span style={{ marginLeft: 16 }}>
            Page: {page} / {totalPages}
            <button onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page === 1} style={{ marginLeft: 8 }}>Prev</button>
            <button onClick={() => setPage(p => Math.min(totalPages, p + 1))} disabled={page === totalPages} style={{ marginLeft: 4 }}>Next</button>
          </span>
        )}
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
            {services.map((svc) => {
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
  );
}

export default App;

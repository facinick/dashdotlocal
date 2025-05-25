import { useEffect, useState } from 'react';
import { z } from 'zod';
import './App.css';

const ServiceSchema = z.object({
  port: z.number(),
  status: z.string(),
  process: z.string().optional(),
  pid: z.number().optional(),
  user: z.string().optional(),
  protocol: z.string().optional(),
  local_address: z.string().optional(),
  fd: z.string().optional(),
  type_field: z.string().optional(),
  device: z.string().optional(),
  size_off: z.string().optional(),
  node: z.string().optional(),
  command_line: z.string().optional(),
  exe_path: z.string().optional(),
  start_time: z.string().optional(),
  ppid: z.number().optional(),
});

const ServicesSchema = z.array(ServiceSchema);

export type Service = z.infer<typeof ServiceSchema>;

function App() {
  const [services, setServices] = useState<Service[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    fetch('http://localhost:3000/')
      .then((res) => res.json())
      .then((data) => {
        const result = ServicesSchema.safeParse(data);
        if (result.success) {
          setServices(result.data);
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
  }, [])

  return (
    <main className="container">
      <h1>Local Services Dashboard</h1>
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
            </tr>
          </thead>
          <tbody>
            {services.map((svc) => (
              <tr key={svc.port}>
                <td>{svc.port}</td>
                <td>{svc.status}</td>
                <td>{svc.process}</td>
                <td>{svc.pid}</td>
                <td>{svc.user}</td>
                <td>{svc.command_line}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </main>
  )
}

export default App

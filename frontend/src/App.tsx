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

// --- Recognizable Service System ---
export type RecognizedService = {
  id: string;
  label: string;
  matcher: (svc: Service) => boolean;
  color: string; // for tag styling
};

export const recognizedServices: RecognizedService[] = [
  {
    id: 'docker',
    label: 'Docker',
    color: '#2496ed',
    matcher: (svc) =>
      !!(svc.process?.toLowerCase() === 'dockerd' ||
        svc.command_line?.toLowerCase().includes('docker')),
  },
  {
    id: 'vite',
    label: 'Vite',
    color: '#646cff',
    matcher: (svc) =>
      !!(svc.command_line?.toLowerCase().includes('vite') ||
        svc.process?.toLowerCase() === 'vite'),
  },
  {
    id: 'node',
    label: 'Node.js',
    color: '#43853d',
    matcher: (svc) =>
      !!(svc.process?.toLowerCase() === 'node' ||
        svc.command_line?.toLowerCase().includes('node')),
  },
  {
    id: 'mongodb',
    label: 'MongoDB',
    color: '#47a248',
    matcher: (svc) =>
      !!(svc.process?.toLowerCase() === 'mongod' ||
        svc.command_line?.toLowerCase().includes('mongod') ||
        svc.port === 27017),
  },
  {
    id: 'postgres',
    label: 'PostgreSQL',
    color: '#336791',
    matcher: (svc) =>
      !!(svc.process?.toLowerCase().includes('postgres') ||
        svc.command_line?.toLowerCase().includes('postgres') ||
        svc.port === 5432),
  },
  {
    id: 'redis',
    label: 'Redis',
    color: '#d82c20',
    matcher: (svc) =>
      !!(svc.process?.toLowerCase().includes('redis') ||
        svc.command_line?.toLowerCase().includes('redis') ||
        svc.port === 6379),
  },
  {
    id: 'ollama',
    label: 'Ollama',
    color: '#222',
    matcher: (svc) =>
      !!(
        svc.process?.toLowerCase() === 'ollama' ||
        svc.command_line?.toLowerCase().includes('ollama') ||
        svc.port === 11434
      ),
  },
  // Add more as needed...
];

function getRecognizedService(svc: Service): RecognizedService | undefined {
  return recognizedServices.find((rec) => rec.matcher(svc));
}

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
              <th>Type</th>
            </tr>
          </thead>
          <tbody>
            {services.map((svc) => {
              const recognized = getRecognizedService(svc);
              return (
                <tr key={svc.port}>
                  <td>{svc.port}</td>
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

export default App

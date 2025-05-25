import { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';
import { type Service, ServiceSchema } from '../../lib/serviceSchema';


function ServiceDetail() {
  const { port } = useParams();
  const [service, setService] = useState<Service | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!port) return;
    fetch(`http://localhost:3000/${port}`)
      .then(res => res.json())
      .then(data => {
        const result = ServiceSchema.safeParse(data);
        if (result.success) {
          setService(result.data);
          setLoading(false);
        } else {
          setError('Invalid data from server');
          setLoading(false);
        }
      })
      .catch(() => {
        setError('Failed to load service');
        setLoading(false);
      });
  }, [port]);

  return (
    <main className="container">
      <Link to="/">← Back to Dashboard</Link>
      <h1>Service Details</h1>
      {loading && <p>Loading...</p>}
      {error && <p style={{ color: 'red' }}>{error}</p>}
      {service && (
        <table className="services-table">
          <tbody>
            {Object.entries(service).map(([key, value]) => (
              <tr key={key}>
                <td style={{ fontWeight: 600 }}>{key}</td>
                <td>{String(value)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </main>
  );
}

export default ServiceDetail; 
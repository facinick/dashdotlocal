import type { Service } from './serviceSchema';

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

export function getRecognizedService(svc: Service): RecognizedService | undefined {
  return recognizedServices.find((rec) => rec.matcher(svc));
} 
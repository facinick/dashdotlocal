import { getRecognizedService } from './recognizedServices';
import type { Service } from './serviceSchema';

export type SortField =
  | 'port'
  | 'status'
  | 'process'
  | 'pid'
  | 'user'
  | 'command_line'
  | 'start_time'
  | 'recognized';

export type SortDirection = 'asc' | 'desc';

export const sortFields: { value: SortField; label: string }[] = [
  { value: 'port', label: 'Port' },
  { value: 'status', label: 'Status' },
  { value: 'process', label: 'Process' },
  { value: 'pid', label: 'PID' },
  { value: 'user', label: 'User' },
  { value: 'command_line', label: 'Command' },
  { value: 'start_time', label: 'Started Time' },
  { value: 'recognized', label: 'Recognized Service' },
];

export const sortDirections: { value: SortDirection; label: string }[] = [
  { value: 'asc', label: 'Ascending' },
  { value: 'desc', label: 'Descending' },
];

export function sortServices(
  services: Service[],
  field: SortField,
  direction: SortDirection
): Service[] {
  const sorted = [...services].sort((a, b) => {
    let aValue: string | number | boolean;
    let bValue: string | number | boolean;
    if (field === 'recognized') {
      aValue = !!getRecognizedService(a);
      bValue = !!getRecognizedService(b);
    } else {
      aValue = a[field] ?? '';
      bValue = b[field] ?? '';
    }
    if (typeof aValue === 'string' && typeof bValue === 'string') {
      return aValue.localeCompare(bValue);
    }
    if (typeof aValue === 'number' && typeof bValue === 'number') {
      return aValue - bValue;
    }
    if (typeof aValue === 'boolean' && typeof bValue === 'boolean') {
      return Number(aValue) - Number(bValue);
    }
    return 0;
  });
  return direction === 'asc' ? sorted : sorted.reverse();
} 
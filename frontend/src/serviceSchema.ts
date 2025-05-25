import { z } from 'zod';

export const ServiceSchema = z.object({
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

export const ServicesSchema = z.array(ServiceSchema);

export type Service = z.infer<typeof ServiceSchema>; 
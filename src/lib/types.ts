export type InstanceStatus = 'running' | 'stopped' | 'error' | 'creating';

export type DatabaseType = 'postgres' | 'redis' | 'mysql' | 'mongo';

export interface Instance {
  id: string;
  name: string;
  database_type: DatabaseType;
  image: string;
  tag: string;
  port: number;
  root_password: string;
  status: InstanceStatus;
  created_at: string;
  volume_path?: string;
}

export interface CreateInstanceRequest {
  name: string;
  database_type: DatabaseType;
  image: string;
  tag: string;
  password: string;
  port?: number;
}

export interface ImageTag {
  name: string;
}

export interface SupportedImage {
  id: string;
  name: string;
  hubName: string;
  description: string;
  default_port: number;
}

export interface PullProgress {
  id: string;
  status: string;
  progress?: string;
  progress_detail?: {
    current?: number;
    total?: number;
  };
}

export const SUPPORTED_IMAGES: SupportedImage[] = [
  { 
    id: 'postgres', 
    name: 'PostgreSQL', 
    hubName: 'library/postgres',
    description: 'Advanced open source database',
    default_port: 5432
  },
  { 
    id: 'redis', 
    name: 'Redis', 
    hubName: 'library/redis',
    description: 'In-memory data structure store',
    default_port: 6379
  },
  { 
    id: 'mysql', 
    name: 'MySQL', 
    hubName: 'library/mysql',
    description: 'Popular relational database',
    default_port: 3306
  },
  { 
    id: 'mongo', 
    name: 'MongoDB', 
    hubName: 'library/mongo',
    description: 'NoSQL document database',
    default_port: 27017
  },
];

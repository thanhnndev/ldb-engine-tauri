export interface ImageTag {
  name: string;
}

export interface SupportedImage {
  id: string;
  name: string;
  hubName: string;
  description: string;
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
    description: 'Advanced open source database'
  },
  { 
    id: 'redis', 
    name: 'Redis', 
    hubName: 'library/redis',
    description: 'In-memory data structure store'
  },
  { 
    id: 'mysql', 
    name: 'MySQL', 
    hubName: 'library/mysql',
    description: 'Popular relational database'
  },
  { 
    id: 'mongo', 
    name: 'MongoDB', 
    hubName: 'library/mongo',
    description: 'NoSQL document database'
  },
];

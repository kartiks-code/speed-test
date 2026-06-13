import { describe, it, expect, mock, beforeAll, beforeEach } from 'bun:test';

// Queue of results that the mock sql will return in order
const sqlResults: unknown[][] = [];

// Mock the tagged-template sql client before importing the repository.
// postgres.js uses tagged template literals: sql`...` → Promise<Row[]>
// A plain function mock works because tagged templates call it as fn(strings, ...values).
const mockSql = mock((_strings: TemplateStringsArray, ..._values: unknown[]) => {
  const next = sqlResults.shift();
  return Promise.resolve(next ?? []);
});

mock.module('../../db/client', () => ({ default: mockSql }));

// Dynamic import so the mock above is already in place when the module loads
let petRepo: typeof import('../../db/petRepository');
beforeAll(async () => {
  petRepo = (await import('../../db/petRepository')) as typeof import('../../db/petRepository');
});

beforeEach(() => {
  sqlResults.length = 0;
  mockSql.mockClear();
});

describe('petRepository', () => {
  describe('add', () => {
    it('calls nextval and inserts with returned id', async () => {
      sqlResults.push([{ id: BigInt(42) }]); // nextval
      sqlResults.push([]);                    // insert
      const result = await petRepo.add({ name: 'Fido', photoUrls: ['http://ex.com/img.jpg'] });
      expect(result.id).toBe(42);
      expect(result.name).toBe('Fido');
    });

    it('uses supplied id without calling nextval separately', async () => {
      sqlResults.push([{ id: BigInt(99) }]); // nextval (called but petId overrides)
      sqlResults.push([]);                    // insert
      const result = await petRepo.add({ id: 7, name: 'Buddy', photoUrls: [] });
      expect(result.id).toBe(7);
    });

    it('returns pet with status when provided', async () => {
      sqlResults.push([{ id: BigInt(1) }]);
      sqlResults.push([]);
      const result = await petRepo.add({ name: 'Rex', photoUrls: [], status: 'sold' });
      expect(result.status).toBe('sold');
    });
  });

  describe('update', () => {
    it('returns null when no rows updated', async () => {
      sqlResults.push([]); // UPDATE returns empty
      const result = await petRepo.update({ id: 999, name: 'Ghost', photoUrls: [] });
      expect(result).toBeNull();
    });

    it('returns null when pet has no id', async () => {
      const result = await petRepo.update({ name: 'NoId', photoUrls: [] });
      expect(result).toBeNull();
    });

    it('returns the updated pet on success', async () => {
      sqlResults.push([{
        id: 5, name: 'Updated', category: null, photo_urls: '[]', tags: null, status: 'pending',
      }]);
      const result = await petRepo.update({ id: 5, name: 'Updated', photoUrls: [], status: 'pending' });
      expect(result).not.toBeNull();
      expect(result!.name).toBe('Updated');
      expect(result!.status).toBe('pending');
    });
  });

  describe('findByStatus', () => {
    it('returns empty array when no pets found', async () => {
      sqlResults.push([]);
      const result = await petRepo.findByStatus('available');
      expect(result).toEqual([]);
    });

    it('maps rows to Pet objects', async () => {
      sqlResults.push([
        { id: 1, name: 'Fido', category: null, photo_urls: '[]', tags: null, status: 'available' },
      ]);
      const result = await petRepo.findByStatus('available');
      expect(result).toHaveLength(1);
      expect(result[0].name).toBe('Fido');
      expect(result[0].status).toBe('available');
    });

    it('parses category JSON from text column', async () => {
      sqlResults.push([
        { id: 2, name: 'Cat', category: '{"id":1,"name":"cats"}', photo_urls: '[]', tags: null, status: 'available' },
      ]);
      const [pet] = await petRepo.findByStatus('available');
      expect(pet.category).toEqual({ id: 1, name: 'cats' });
    });
  });

  describe('findById', () => {
    it('returns null when pet not found', async () => {
      sqlResults.push([]);
      const result = await petRepo.findById(1);
      expect(result).toBeNull();
    });

    it('returns parsed pet when found', async () => {
      sqlResults.push([
        { id: 3, name: 'Dog', category: null, photo_urls: '["http://ex.com"]', tags: '[]', status: 'sold' },
      ]);
      const result = await petRepo.findById(3);
      expect(result).not.toBeNull();
      expect(result!.id).toBe(3);
      expect(result!.photoUrls).toEqual(['http://ex.com']);
      expect(result!.tags).toEqual([]);
    });
  });

  describe('findByTags', () => {
    it('returns all pets when tags is empty', async () => {
      sqlResults.push([
        { id: 1, name: 'All', category: null, photo_urls: '[]', tags: null, status: 'available' },
      ]);
      const result = await petRepo.findByTags([]);
      expect(result).toHaveLength(1);
    });

    it('queries by tags when tags provided', async () => {
      sqlResults.push([]);
      const result = await petRepo.findByTags(['dogs']);
      expect(result).toEqual([]);
    });
  });

  describe('updateWithForm', () => {
    it('returns false when pet not found', async () => {
      sqlResults.push([]); // findById → empty
      const result = await petRepo.updateWithForm(999);
      expect(result).toBe(false);
    });

    it('returns true when pet updated', async () => {
      sqlResults.push([
        { id: 1, name: 'OldName', category: null, photo_urls: '[]', tags: null, status: 'available' },
      ]); // findById
      sqlResults.push([]); // UPDATE
      const result = await petRepo.updateWithForm(1, 'NewName', 'sold');
      expect(result).toBe(true);
    });
  });

  describe('remove', () => {
    it('returns false when pet not found', async () => {
      sqlResults.push(Object.assign([], { count: 0 }));
      const result = await petRepo.remove(999);
      expect(result).toBe(false);
    });

    it('returns true when pet deleted', async () => {
      sqlResults.push(Object.assign([], { count: 1 }));
      const result = await petRepo.remove(1);
      expect(result).toBe(true);
    });
  });

  describe('addPhoto', () => {
    it('returns null when pet does not exist', async () => {
      sqlResults.push([]); // SELECT 1 FROM pet → not found
      const result = await petRepo.addPhoto(999, Buffer.from('data'));
      expect(result).toBeNull();
    });

    it('returns ApiResponse with byte count on success', async () => {
      sqlResults.push([{ '?column?': 1 }]); // SELECT 1 FROM pet → found
      sqlResults.push([]);                    // INSERT
      const content = Buffer.from('hello world');
      const result = await petRepo.addPhoto(1, content);
      expect(result).not.toBeNull();
      expect(result!.code).toBe(200);
      expect(result!.message).toContain(`${content.length} bytes`);
    });
  });
});

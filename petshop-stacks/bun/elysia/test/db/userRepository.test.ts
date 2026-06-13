import { describe, it, expect, mock, beforeAll, beforeEach } from 'bun:test';

const sqlResults: unknown[][] = [];

const mockSql = mock((_strings: TemplateStringsArray, ..._values: unknown[]) => {
  const next = sqlResults.shift();
  return Promise.resolve(next ?? []);
});

mock.module('../../db/client', () => ({ default: mockSql }));

let userRepo: typeof import('../../db/userRepository');
beforeAll(async () => {
  userRepo = (await import('../../db/userRepository')) as typeof import('../../db/userRepository');
});

beforeEach(() => {
  sqlResults.length = 0;
  mockSql.mockClear();
});

const userRow = {
  id: 1,
  username: 'jdoe',
  first_name: 'John',
  last_name: 'Doe',
  email: 'jdoe@example.com',
  password: 'secret',
  phone: '555-0100',
  user_status: 1,
};

describe('userRepository', () => {
  describe('createUser', () => {
    it('upserts and returns the created user', async () => {
      sqlResults.push([{ id: BigInt(1) }]); // nextval
      sqlResults.push([userRow]);             // INSERT RETURNING
      const result = await userRepo.createUser({
        username: 'jdoe', firstName: 'John', lastName: 'Doe',
        email: 'jdoe@example.com', password: 'secret',
      });
      expect(result.username).toBe('jdoe');
      expect(result.firstName).toBe('John');
    });
  });

  describe('createUsersWithList', () => {
    it('throws when all users lack username', async () => {
      expect(() =>
        userRepo.createUsersWithList([{ firstName: 'NoUser' }])
      ).toThrow();
    });

    it('returns last successfully upserted user', async () => {
      sqlResults.push([{ id: BigInt(2) }]); // nextval for user 1
      sqlResults.push([userRow]);             // INSERT for user 1
      sqlResults.push([{ id: BigInt(3) }]); // nextval for user 2
      sqlResults.push([{ ...userRow, id: 3, username: 'jane' }]); // INSERT for user 2
      const result = await userRepo.createUsersWithList([
        { username: 'jdoe' },
        { username: 'jane' },
      ]);
      expect(result.username).toBe('jane');
    });
  });

  describe('findByUsername', () => {
    it('returns null when user not found', async () => {
      sqlResults.push([]);
      const result = await userRepo.findByUsername('nobody');
      expect(result).toBeNull();
    });

    it('maps row to User object', async () => {
      sqlResults.push([userRow]);
      const result = await userRepo.findByUsername('jdoe');
      expect(result).not.toBeNull();
      expect(result!.username).toBe('jdoe');
      expect(result!.lastName).toBe('Doe');
    });
  });

  describe('updateUser', () => {
    it('returns false when user not found', async () => {
      sqlResults.push([]); // UPDATE returns no rows
      const result = await userRepo.updateUser('nobody', { firstName: 'X' });
      expect(result).toBe(false);
    });

    it('returns true when user updated', async () => {
      sqlResults.push([{ id: 1 }]); // UPDATE RETURNING
      const result = await userRepo.updateUser('jdoe', { firstName: 'Jane' });
      expect(result).toBe(true);
    });
  });

  describe('remove', () => {
    it('returns false when user not found', async () => {
      sqlResults.push(Object.assign([], { count: 0 }));
      const result = await userRepo.remove('nobody');
      expect(result).toBe(false);
    });

    it('returns true when user deleted', async () => {
      sqlResults.push(Object.assign([], { count: 1 }));
      const result = await userRepo.remove('jdoe');
      expect(result).toBe(true);
    });
  });

  describe('login', () => {
    it('returns null when credentials do not match', async () => {
      sqlResults.push([]);
      const result = await userRepo.login('jdoe', 'wrong');
      expect(result).toBeNull();
    });

    it('returns user when credentials match', async () => {
      sqlResults.push([userRow]);
      const result = await userRepo.login('jdoe', 'secret');
      expect(result).not.toBeNull();
      expect(result!.username).toBe('jdoe');
    });
  });
});

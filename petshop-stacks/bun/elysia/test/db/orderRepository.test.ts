import { describe, it, expect, mock, beforeAll, beforeEach } from 'bun:test';

const sqlResults: unknown[][] = [];

const mockSql = mock((_strings: TemplateStringsArray, ..._values: unknown[]) => {
  const next = sqlResults.shift();
  return Promise.resolve(next ?? []);
});

mock.module('../../db/client', () => ({ default: mockSql }));

let orderRepo: typeof import('../../db/orderRepository');
beforeAll(async () => {
  orderRepo = (await import('../../db/orderRepository')) as typeof import('../../db/orderRepository');
});

beforeEach(() => {
  sqlResults.length = 0;
  mockSql.mockClear();
});

describe('orderRepository', () => {
  describe('placeOrder', () => {
    it('calls nextval and inserts order, returning it', async () => {
      sqlResults.push([{ id: BigInt(10) }]); // nextval
      sqlResults.push([{                      // INSERT RETURNING
        id: 10, pet_id: 1, quantity: 2, ship_date: '2026-01-01', status: 'placed', complete: false,
      }]);
      const result = await orderRepo.placeOrder({ petId: 1, quantity: 2, status: 'placed' });
      expect(result.id).toBe(10);
      expect(result.petId).toBe(1);
      expect(result.status).toBe('placed');
    });

    it('uses supplied id when provided', async () => {
      sqlResults.push([{ id: BigInt(99) }]); // nextval (not used since id provided)
      sqlResults.push([{
        id: 5, pet_id: 2, quantity: 1, ship_date: null, status: 'approved', complete: true,
      }]);
      const result = await orderRepo.placeOrder({ id: 5, petId: 2, quantity: 1, status: 'approved', complete: true });
      expect(result.id).toBe(5);
      expect(result.complete).toBe(true);
    });
  });

  describe('findById', () => {
    it('returns null when order not found', async () => {
      sqlResults.push([]);
      const result = await orderRepo.findById(42);
      expect(result).toBeNull();
    });

    it('returns mapped order when found', async () => {
      sqlResults.push([{
        id: 7, pet_id: 3, quantity: 5, ship_date: '2026-06-01', status: 'delivered', complete: true,
      }]);
      const result = await orderRepo.findById(7);
      expect(result).not.toBeNull();
      expect(result!.id).toBe(7);
      expect(result!.quantity).toBe(5);
      expect(result!.status).toBe('delivered');
    });
  });

  describe('remove', () => {
    it('returns false when order not found', async () => {
      sqlResults.push(Object.assign([], { count: 0 }));
      const result = await orderRepo.remove(999);
      expect(result).toBe(false);
    });

    it('returns true when order deleted', async () => {
      sqlResults.push(Object.assign([], { count: 1 }));
      const result = await orderRepo.remove(1);
      expect(result).toBe(true);
    });
  });

  describe('getInventory', () => {
    it('returns empty object when no pets', async () => {
      sqlResults.push([]);
      const result = await orderRepo.getInventory();
      expect(result).toEqual({});
    });

    it('maps status counts to object', async () => {
      sqlResults.push([
        { status: 'available', cnt: 5 },
        { status: 'sold', cnt: 3 },
      ]);
      const result = await orderRepo.getInventory();
      expect(result).toEqual({ available: 5, sold: 3 });
    });
  });
});

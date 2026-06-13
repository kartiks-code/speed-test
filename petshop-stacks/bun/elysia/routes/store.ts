import { Elysia } from 'elysia';
import * as orderRepo from '../db/orderRepository';

export const storeRoutes = new Elysia()
  .get('/store/inventory', async () => {
    return orderRepo.getInventory();
  })
  .post('/store/order', async ({ body, set }) => {
    const order = body as Record<string, unknown>;
    if (!order) {
      set.status = 400;
      return { message: 'Order body is required' };
    }
    return orderRepo.placeOrder(order as Parameters<typeof orderRepo.placeOrder>[0]);
  })
  .get('/store/order/:orderId', async ({ params, set }) => {
    const id = parseInt(params.orderId, 10);
    if (isNaN(id) || id <= 0) {
      set.status = 400;
      return { message: 'Invalid ID supplied' };
    }
    const order = await orderRepo.findById(id);
    if (!order) {
      set.status = 404;
      return { message: 'Order not found' };
    }
    return order;
  })
  .delete('/store/order/:orderId', async ({ params, set }) => {
    const id = parseInt(params.orderId, 10);
    if (isNaN(id) || id <= 0) {
      set.status = 400;
      return { message: 'Invalid ID supplied' };
    }
    const ok = await orderRepo.remove(id);
    if (!ok) {
      set.status = 404;
      return { message: 'Order not found' };
    }
    set.status = 200;
    return { message: 'Order deleted' };
  });

import { Elysia } from 'elysia';
import { petRoutes } from './routes/pet';
import { storeRoutes } from './routes/store';
import { userRoutes } from './routes/user';

export function buildApp() {
  return new Elysia({ prefix: '/api/v3' })
    .use(petRoutes)
    .use(storeRoutes)
    .use(userRoutes);
}

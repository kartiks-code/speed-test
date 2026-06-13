import { Elysia } from 'elysia';
import * as userRepo from '../db/userRepository';
import type { User } from '../models';
import { randomBytes } from 'crypto';

export const userRoutes = new Elysia()
  .post('/user', async ({ body, set }) => {
    const user = body as User;
    if (!user?.username) {
      set.status = 400;
      return { message: 'Username is required' };
    }
    return userRepo.createUser(user);
  })
  .post('/user/createWithList', async ({ body, set }) => {
    const users = body as User[];
    if (!Array.isArray(users) || users.length === 0) {
      set.status = 400;
      return { message: 'User list is required' };
    }
    try {
      return await userRepo.createUsersWithList(users);
    } catch {
      set.status = 400;
      return { message: 'No valid users (all missing username)' };
    }
  })
  .get('/user/login', async ({ query, set }) => {
    const { username, password } = query as { username?: string; password?: string };
    if (!username || !password) {
      set.status = 400;
      return { message: 'Username and password are required' };
    }
    const user = await userRepo.login(username, password);
    if (!user) {
      set.status = 400;
      return { message: 'Invalid username or password' };
    }
    return `token:${randomBytes(16).toString('hex')}`;
  })
  .get('/user/logout', () => {
    // Stateless no-op
    return { message: 'OK' };
  })
  .get('/user/:username', async ({ params, set }) => {
    const user = await userRepo.findByUsername(params.username);
    if (!user) {
      set.status = 404;
      return { message: 'User not found' };
    }
    return user;
  })
  .put('/user/:username', async ({ params, body, set }) => {
    const user = body as User;
    if (!user) {
      set.status = 400;
      return { message: 'User body is required' };
    }
    const ok = await userRepo.updateUser(params.username, user);
    if (!ok) {
      set.status = 404;
      return { message: 'User not found' };
    }
    return { message: 'User updated' };
  })
  .delete('/user/:username', async ({ params, set }) => {
    const ok = await userRepo.remove(params.username);
    if (!ok) {
      set.status = 404;
      return { message: 'User not found' };
    }
    set.status = 200;
    return { message: 'User deleted' };
  });

import { Elysia } from 'elysia';
import * as petRepo from '../db/petRepository';

export const petRoutes = new Elysia()
  .post('/pet', async ({ body, set }) => {
    const pet = body as Record<string, unknown>;
    if (!pet?.name || !Array.isArray(pet.photoUrls)) {
      set.status = 405;
      return { message: 'Invalid input' };
    }
    return petRepo.add(pet as Parameters<typeof petRepo.add>[0]);
  })
  .put('/pet', async ({ body, set }) => {
    const pet = body as Record<string, unknown>;
    if (!pet?.name || !Array.isArray(pet.photoUrls)) {
      set.status = 405;
      return { message: 'Invalid input' };
    }
    if (pet.id === undefined) {
      set.status = 400;
      return { message: 'Pet ID is required for update' };
    }
    const result = await petRepo.update(pet as Parameters<typeof petRepo.update>[0]);
    if (!result) {
      set.status = 404;
      return { message: 'Pet not found' };
    }
    return result;
  })
  .get('/pet/findByStatus', async ({ query, set }) => {
    const status = (query as Record<string, string>).status || 'available';
    const validStatuses = ['available', 'pending', 'sold'];
    if (!validStatuses.includes(status)) {
      set.status = 400;
      return { message: 'Invalid status value' };
    }
    return petRepo.findByStatus(status);
  })
  .get('/pet/findByTags', async ({ query }) => {
    const raw = (query as Record<string, string | string[]>).tags;
    const tags = Array.isArray(raw) ? raw : raw ? [raw] : [];
    return petRepo.findByTags(tags);
  })
  .get('/pet/:petId', async ({ params, set }) => {
    const id = parseInt(params.petId, 10);
    if (isNaN(id)) {
      set.status = 400;
      return { message: 'Invalid ID supplied' };
    }
    const pet = await petRepo.findById(id);
    if (!pet) {
      set.status = 404;
      return { message: 'Pet not found' };
    }
    return pet;
  })
  .post('/pet/:petId', async ({ params, body, set }) => {
    const id = parseInt(params.petId, 10);
    if (isNaN(id)) {
      set.status = 405;
      return { message: 'Invalid input' };
    }
    const form = body as Record<string, string> | null;
    const ok = await petRepo.updateWithForm(id, form?.name, form?.status);
    if (!ok) {
      set.status = 404;
      return { message: 'Pet not found' };
    }
    return { code: 200, type: 'unknown', message: String(id) };
  })
  .delete('/pet/:petId', async ({ params, set }) => {
    const id = parseInt(params.petId, 10);
    if (isNaN(id)) {
      set.status = 400;
      return { message: 'Invalid pet value' };
    }
    const ok = await petRepo.remove(id);
    if (!ok) {
      set.status = 404;
      return { message: 'Pet not found' };
    }
    set.status = 200;
    return { message: 'Pet deleted' };
  })
  .post('/pet/:petId/uploadImage', async ({ params, body, set, request }) => {
    const id = parseInt(params.petId, 10);
    if (isNaN(id)) {
      set.status = 400;
      return { message: 'Invalid pet ID' };
    }

    let bytes: Buffer;
    if (body instanceof ArrayBuffer) {
      bytes = Buffer.from(body);
    } else if (Buffer.isBuffer(body)) {
      bytes = body;
    } else if (body instanceof Uint8Array) {
      bytes = Buffer.from(body);
    } else {
      // Fall back to reading the raw request body
      const raw = await request.arrayBuffer();
      bytes = Buffer.from(raw);
    }

    const contentType = request.headers.get('content-type') || 'application/octet-stream';
    const result = await petRepo.addPhoto(id, bytes, contentType);
    if (!result) {
      set.status = 404;
      return { message: 'Pet not found' };
    }
    return result;
  });

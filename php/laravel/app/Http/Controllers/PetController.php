<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Repositories\PetstoreRepositoryInterface;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Http\Response;

class PetController extends Controller
{
    public function __construct(private readonly PetstoreRepositoryInterface $repo) {}

    public function addPet(Request $request): JsonResponse
    {
        $data = $request->json()->all();
        if (empty($data['name']) || !isset($data['photoUrls'])) {
            return response()->json(['code' => '400', 'message' => 'Invalid input'], 400);
        }
        $pet = $this->repo->addPet($data);
        return response()->json($pet, 200);
    }

    public function updatePet(Request $request): JsonResponse
    {
        $data = $request->json()->all();
        if (empty($data['id'])) {
            return response()->json(['code' => '400', 'message' => 'Invalid ID supplied'], 400);
        }
        $pet = $this->repo->updatePet($data);
        if ($pet === null) {
            return response()->json(['code' => '404', 'message' => 'Pet not found'], 404);
        }
        return response()->json($pet, 200);
    }

    public function findPetsByStatus(Request $request): JsonResponse
    {
        $status = $request->query('status', 'available');
        if (!in_array($status, ['available', 'pending', 'sold'], true)) {
            return response()->json(['code' => '400', 'message' => 'Invalid status value'], 400);
        }
        return response()->json($this->repo->findPetsByStatus($status), 200);
    }

    public function findPetsByTags(Request $request): JsonResponse
    {
        $tags = $request->query('tags', []);
        if (is_string($tags)) {
            $tags = [$tags];
        }
        return response()->json($this->repo->findPetsByTags((array) $tags), 200);
    }

    public function getPetById(Request $request, int $petId): JsonResponse
    {
        $pet = $this->repo->getPetById($petId);
        if ($pet === null) {
            return response()->json(['code' => '404', 'message' => 'Pet not found'], 404);
        }
        return response()->json($pet, 200);
    }

    public function updatePetWithForm(Request $request, int $petId): JsonResponse
    {
        $name   = $request->query('name');
        $status = $request->query('status');
        $ok = $this->repo->updatePetWithForm($petId, $name, $status);
        if (!$ok) {
            return response()->json(['code' => '400', 'message' => 'Invalid input'], 400);
        }
        return response()->json(null, 200);
    }

    public function deletePet(Request $request, int $petId): JsonResponse
    {
        $ok = $this->repo->deletePet($petId);
        if (!$ok) {
            return response()->json(['code' => '400', 'message' => 'Invalid pet value'], 400);
        }
        return response()->json(null, 200);
    }

    public function uploadFile(Request $request, int $petId): JsonResponse
    {
        if ($this->repo->getPetById($petId) === null) {
            return response()->json(['code' => '404', 'message' => 'Pet not found'], 404);
        }
        $content = $request->getContent();
        $bytes   = $this->repo->uploadFile($petId, $content);
        return response()->json([
            'code'    => 200,
            'type'    => 'unknown',
            'message' => "File uploaded, {$bytes} bytes stored",
        ], 200);
    }
}

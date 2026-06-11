<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Repositories\PetstoreRepositoryInterface;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class UserController extends Controller
{
    public function __construct(private readonly PetstoreRepositoryInterface $repo) {}

    public function createUser(Request $request): JsonResponse
    {
        $data = $request->json()->all();
        $user = $this->repo->createUser($data);
        return response()->json($user, 200);
    }

    public function createUsersWithListInput(Request $request): JsonResponse
    {
        $users = $request->json()->all();
        if (!is_array($users) || empty($users)) {
            return response()->json(['code' => '400', 'message' => 'Invalid input'], 400);
        }
        $last = null;
        foreach ($users as $u) {
            $last = $this->repo->createUser($u);
        }
        return response()->json($last, 200);
    }

    public function getUserByName(Request $request, string $username): JsonResponse
    {
        $user = $this->repo->getUserByName($username);
        if ($user === null) {
            return response()->json(['code' => '404', 'message' => 'User not found'], 404);
        }
        return response()->json($user, 200);
    }

    public function updateUser(Request $request, string $username): JsonResponse
    {
        $data = $request->json()->all();
        $ok   = $this->repo->updateUser($username, $data);
        if (!$ok) {
            return response()->json(['code' => '404', 'message' => 'User not found'], 404);
        }
        return response()->json(null, 200);
    }

    public function deleteUser(Request $request, string $username): JsonResponse
    {
        $ok = $this->repo->deleteUser($username);
        if (!$ok) {
            return response()->json(['code' => '404', 'message' => 'User not found'], 404);
        }
        return response()->json(null, 200);
    }

    public function loginUser(Request $request): JsonResponse
    {
        $username = $request->query('username', '');
        $password = $request->query('password', '');
        $token    = $this->repo->loginUser($username, $password);
        if ($token === null) {
            return response()->json(['code' => '400', 'message' => 'Invalid username/password'], 400);
        }
        return response()->json($token, 200)
            ->header('X-Rate-Limit', '5000')
            ->header('X-Expires-After', gmdate('Y-m-d\TH:i:s\Z', time() + 3600));
    }

    public function logoutUser(): JsonResponse
    {
        return response()->json(null, 200);
    }
}

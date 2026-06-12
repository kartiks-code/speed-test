<?php

declare(strict_types=1);

namespace App\Http\Controllers;

use App\Repositories\PetstoreRepositoryInterface;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class StoreController extends Controller
{
    public function __construct(private readonly PetstoreRepositoryInterface $repo) {}

    public function getInventory(): JsonResponse
    {
        return response()->json($this->repo->getInventory(), 200);
    }

    public function placeOrder(Request $request): JsonResponse
    {
        $data  = $request->json()->all();
        $order = $this->repo->placeOrder($data);
        return response()->json($order, 200);
    }

    public function getOrderById(Request $request, int $orderId): JsonResponse
    {
        $order = $this->repo->getOrderById($orderId);
        if ($order === null) {
            return response()->json(['code' => '404', 'message' => 'Order not found'], 404);
        }
        return response()->json($order, 200);
    }

    public function deleteOrder(Request $request, int $orderId): JsonResponse
    {
        $ok = $this->repo->deleteOrder($orderId);
        if (!$ok) {
            return response()->json(['code' => '404', 'message' => 'Order not found'], 404);
        }
        return response()->json(null, 200);
    }
}
